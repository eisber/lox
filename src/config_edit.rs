//! Config XML editing engine — DOM-based modification of .Loxone config files.
//!
//! Provides structured XML editing with proper DOM manipulation and BOM-aware
//! write-back. Element selectors support matching by Title, Type, UUID, or gid.
//!
//! ## Element Selector Syntax
//!
//! ```text
//!   "My Control"       — match by Title (case-insensitive contains)
//!   "Type:WeatherData" — match all elements of a type
//!   "uuid:abc-123"     — match by UUID
//!   "gid:Mqtt"         — match by gid attribute
//! ```

use anyhow::{Context, Result, bail};
use std::collections::HashMap;
use std::io::{BufReader, Cursor};
use xmltree::Element;

const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

/// A Loxone config file loaded for editing.
pub struct ConfigEditor {
    pub root: Element,
    had_bom: bool,
    had_crlf: bool,
}

impl ConfigEditor {
    /// Load a .Loxone XML file for editing.
    pub fn load(data: &[u8]) -> Result<Self> {
        let had_bom = data.starts_with(UTF8_BOM);
        let had_crlf = data.windows(2).any(|w| w == b"\r\n");
        let xml_data = if had_bom { &data[3..] } else { data };
        let reader = BufReader::new(Cursor::new(xml_data));
        let root = Element::parse(reader).context("Failed to parse Loxone XML")?;
        Ok(ConfigEditor {
            root,
            had_bom,
            had_crlf,
        })
    }

    /// Write the edited XML back to bytes, preserving BOM and line endings.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();

        // Write XML declaration
        buf.extend_from_slice(b"<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");

        // Write DOM tree
        let config = xmltree::EmitterConfig::new()
            .perform_indent(true)
            .indent_string("\t")
            .write_document_declaration(false);
        self.root
            .write_with_config(&mut buf, config)
            .context("Failed to write XML")?;

        // Post-process: restore BOM
        if self.had_bom {
            let mut result = Vec::with_capacity(3 + buf.len());
            result.extend_from_slice(UTF8_BOM);
            result.extend_from_slice(&buf);
            buf = result;
        }

        // Post-process: restore CRLF line endings
        if self.had_crlf {
            let s = String::from_utf8(buf).context("XML is not valid UTF-8")?;
            buf = s.replace('\n', "\r\n").into_bytes();
        }

        Ok(buf)
    }

    /// Find all elements matching a selector.
    pub fn find_elements(&self, selector: &str) -> Vec<Vec<usize>> {
        let mut results = Vec::new();
        self.find_recursive(&self.root, selector, &mut Vec::new(), &mut results);
        results
    }

    fn find_recursive(
        &self,
        elem: &Element,
        selector: &str,
        path: &mut Vec<usize>,
        results: &mut Vec<Vec<usize>>,
    ) {
        if matches_selector(elem, selector) {
            results.push(path.clone());
        }
        for (i, child) in elem.children.iter().enumerate() {
            if let Some(child_elem) = child.as_element() {
                path.push(i);
                self.find_recursive(child_elem, selector, path, results);
                path.pop();
            }
        }
    }

    /// Get a mutable reference to an element by path.
    pub fn get_element_mut(&mut self, path: &[usize]) -> &mut Element {
        let mut current = &mut self.root;
        for &idx in path {
            current = current.children[idx].as_mut_element().unwrap();
        }
        current
    }

    /// Get a reference to an element by path.
    fn get_element(&self, path: &[usize]) -> &Element {
        let mut current = &self.root;
        for &idx in path {
            current = current.children[idx].as_element().unwrap();
        }
        current
    }

    /// Require exactly one matching element. Returns the path.
    pub fn require_one(&self, selector: &str) -> Result<Vec<usize>> {
        let matches = self.find_elements(selector);
        match matches.len() {
            0 => bail!("No element matches selector '{}'", selector),
            1 => Ok(matches.into_iter().next().unwrap()),
            n => bail!(
                "Selector '{}' matches {} elements (expected 1). Use a more specific selector.",
                selector,
                n
            ),
        }
    }

    /// Set a property in an element's SET block.
    /// Creates the SET block and property tag if they don't exist.
    pub fn set_property(
        &mut self,
        selector: &str,
        prop_name: &str,
        value: &str,
        type_code: &str,
    ) -> Result<String> {
        let path = self.require_one(selector)?;
        let elem = self.get_element_mut(&path);
        let title = elem.attributes.get("Title").cloned().unwrap_or_default();

        // Find or create SET child
        let set_idx = elem
            .children
            .iter()
            .position(|c| c.as_element().map(|e| e.name == "SET").unwrap_or(false));

        let set_elem = if let Some(idx) = set_idx {
            elem.children[idx].as_mut_element().unwrap()
        } else {
            let set = Element::new("SET");
            elem.children
                .push(xmltree::XMLNode::Element(set));
            let last = elem.children.len() - 1;
            elem.children[last].as_mut_element().unwrap()
        };

        // Find or create property tag
        let prop_idx = set_elem
            .children
            .iter()
            .position(|c| c.as_element().map(|e| e.name == prop_name).unwrap_or(false));

        if let Some(idx) = prop_idx {
            let prop = set_elem.children[idx].as_mut_element().unwrap();
            let old_val = prop.attributes.get("v").cloned().unwrap_or_default();
            prop.attributes.insert("t".to_string(), type_code.to_string());
            prop.attributes.insert("v".to_string(), value.to_string());
            Ok(format!(
                "Updated {}.{}: '{}' → '{}'",
                title, prop_name, old_val, value
            ))
        } else {
            let mut prop = Element::new(prop_name);
            prop.attributes
                .insert("t".to_string(), type_code.to_string());
            prop.attributes
                .insert("v".to_string(), value.to_string());
            set_elem
                .children
                .push(xmltree::XMLNode::Element(prop));
            Ok(format!("Created {}.{} = '{}'", title, prop_name, value))
        }
    }

    /// Set an attribute on an element.
    pub fn set_attribute(
        &mut self,
        selector: &str,
        attr_name: &str,
        value: &str,
    ) -> Result<String> {
        let path = self.require_one(selector)?;
        let elem = self.get_element_mut(&path);
        let title = elem.attributes.get("Title").cloned().unwrap_or_default();
        let old = elem
            .attributes
            .insert(attr_name.to_string(), value.to_string());
        Ok(format!(
            "Set {}.{}: {} → '{}'",
            title,
            attr_name,
            old.as_deref().unwrap_or("(none)"),
            value
        ))
    }

    /// Move all elements matching a type filter to a different room.
    /// Returns the number of elements moved.
    pub fn move_to_room(
        &mut self,
        type_filter: &str,
        target_room_name: &str,
        exclude_types: &[&str],
    ) -> Result<(usize, String)> {
        // First, find the target room UUID
        let room_uuid = self.find_room_uuid(target_room_name)?;

        // Collect paths to matching elements
        let mut paths = Vec::new();
        self.collect_typed_with_iodata(&self.root, type_filter, exclude_types, &mut Vec::new(), &mut paths);

        let count = paths.len();
        for path in &paths {
            let elem = self.get_element_mut(path);
            // Find IoData child and update Pr attribute
            for child in &mut elem.children {
                if let Some(iodata) = child.as_mut_element() {
                    if iodata.name == "IoData" {
                        iodata
                            .attributes
                            .insert("Pr".to_string(), room_uuid.clone());
                    }
                }
            }
        }

        Ok((count, room_uuid))
    }

    pub fn find_room_uuid(&self, room_name: &str) -> Result<String> {
        let lower = room_name.to_lowercase();
        let mut found = Vec::new();
        self.walk_rooms(&self.root, &lower, &mut found);
        match found.len() {
            0 => bail!("Room '{}' not found in config", room_name),
            1 => Ok(found.into_iter().next().unwrap().0),
            _ => {
                // Try exact match
                let exact: Vec<_> = found.iter().filter(|(_uuid, name)| name.to_lowercase() == lower).collect();
                if exact.len() == 1 {
                    Ok(exact[0].0.clone())
                } else {
                    bail!("Multiple rooms match '{}': {:?}", room_name, found.iter().map(|(_, n)| n.as_str()).collect::<Vec<_>>())
                }
            }
        }
    }

    fn walk_rooms(&self, elem: &Element, name_lower: &str, found: &mut Vec<(String, String)>) {
        if elem.name == "C" {
            if let Some(t) = elem.attributes.get("Type") {
                if t == "Place" {
                    if let (Some(uuid), Some(title)) =
                        (elem.attributes.get("U"), elem.attributes.get("Title"))
                    {
                        if title.to_lowercase().contains(name_lower) {
                            found.push((uuid.clone(), title.clone()));
                        }
                    }
                }
            }
        }
        for child in &elem.children {
            if let Some(child_elem) = child.as_element() {
                self.walk_rooms(child_elem, name_lower, found);
            }
        }
    }

    fn collect_typed_with_iodata(
        &self,
        elem: &Element,
        type_filter: &str,
        exclude_types: &[&str],
        path: &mut Vec<usize>,
        results: &mut Vec<Vec<usize>>,
    ) {
        if elem.name == "C" {
            if let Some(t) = elem.attributes.get("Type") {
                if t.eq_ignore_ascii_case(type_filter)
                    && !exclude_types.iter().any(|ex| t.eq_ignore_ascii_case(ex))
                {
                    // Check if has IoData child
                    if elem
                        .children
                        .iter()
                        .any(|c| c.as_element().map(|e| e.name == "IoData").unwrap_or(false))
                    {
                        results.push(path.clone());
                    }
                }
            }
        }
        for (i, child) in elem.children.iter().enumerate() {
            if let Some(child_elem) = child.as_element() {
                path.push(i);
                self.collect_typed_with_iodata(child_elem, type_filter, exclude_types, path, results);
                path.pop();
            }
        }
    }

    /// Describe an element: type, title, UUID, properties, connectors, children.
    pub fn describe(&self, selector: &str) -> Result<ElementDescription> {
        let path = self.require_one(selector)?;
        let elem = self.get_element(path.as_slice());
        self.describe_element(elem)
    }

    fn describe_element(&self, elem: &Element) -> Result<ElementDescription> {
        let mut properties = HashMap::new();
        let mut connectors = Vec::new();
        let mut children = Vec::new();
        let mut room = String::new();
        let mut category = String::new();

        for child in &elem.children {
            if let Some(child_elem) = child.as_element() {
                match child_elem.name.as_str() {
                    "SET" => {
                        for prop_node in &child_elem.children {
                            if let Some(prop) = prop_node.as_element() {
                                let val = prop
                                    .attributes
                                    .get("v")
                                    .cloned()
                                    .unwrap_or_default();
                                let t = prop
                                    .attributes
                                    .get("t")
                                    .cloned()
                                    .unwrap_or_default();
                                properties.insert(
                                    prop.name.clone(),
                                    PropertyValue {
                                        value: val,
                                        type_code: t,
                                    },
                                );
                            }
                        }
                    }
                    "Co" => {
                        connectors.push(ConnectorInfo {
                            kind: child_elem
                                .attributes
                                .get("K")
                                .cloned()
                                .unwrap_or_default(),
                            target: child_elem
                                .attributes
                                .get("U")
                                .cloned()
                                .unwrap_or_default(),
                        });
                    }
                    "IoData" => {
                        room = child_elem
                            .attributes
                            .get("Pr")
                            .cloned()
                            .unwrap_or_default();
                        category = child_elem
                            .attributes
                            .get("Cr")
                            .cloned()
                            .unwrap_or_default();
                    }
                    "C" => {
                        let ct = child_elem
                            .attributes
                            .get("Type")
                            .cloned()
                            .unwrap_or_default();
                        let ct_title = child_elem
                            .attributes
                            .get("Title")
                            .cloned()
                            .unwrap_or_default();
                        children.push(format!("{}: {}", ct, ct_title));
                    }
                    _ => {}
                }
            }
        }

        Ok(ElementDescription {
            element_type: elem
                .attributes
                .get("Type")
                .cloned()
                .unwrap_or_default(),
            title: elem
                .attributes
                .get("Title")
                .cloned()
                .unwrap_or_default(),
            uuid: elem.attributes.get("U").cloned().unwrap_or_default(),
            gid: elem.attributes.get("gid").cloned().unwrap_or_default(),
            room_uuid: room,
            category_uuid: category,
            properties,
            connectors,
            children,
        })
    }

    /// Add a new room (Place element).
    pub fn add_room(&mut self, name: &str) -> Result<String> {
        // Check if room already exists
        let mut existing = Vec::new();
        self.walk_rooms(&self.root, &name.to_lowercase(), &mut existing);
        if !existing.is_empty() {
            bail!("Room '{}' already exists", name);
        }

        // Generate a UUID
        let uuid = format!(
            "{:08x}-{:04x}-{:04x}-ffff000000000000",
            rand::random::<u32>(),
            rand::random::<u16>(),
            rand::random::<u16>()
        );

        // Find PlaceCaption or first Place and insert after
        let mut insert_path = None;
        for (i, child) in self.root.children.iter().enumerate() {
            if let Some(elem) = child.as_element() {
                if elem.name == "C" {
                    if let Some(t) = elem.attributes.get("Type") {
                        if t == "Place" || t == "PlaceCaption" {
                            insert_path = Some(i);
                        }
                    }
                }
            }
        }

        let mut place = Element::new("C");
        place
            .attributes
            .insert("Type".to_string(), "Place".to_string());
        place
            .attributes
            .insert("V".to_string(), "175".to_string());
        place.attributes.insert("U".to_string(), uuid.clone());
        place
            .attributes
            .insert("Title".to_string(), name.to_string());
        place
            .attributes
            .insert("WF".to_string(), "16384".to_string());

        if let Some(idx) = insert_path {
            self.root
                .children
                .insert(idx + 1, xmltree::XMLNode::Element(place));
        } else {
            self.root
                .children
                .push(xmltree::XMLNode::Element(place));
        }

        Ok(uuid)
    }
}

/// Check if an element matches a selector string.
fn matches_selector(elem: &Element, selector: &str) -> bool {
    if elem.name != "C" {
        return false;
    }

    if let Some(uuid) = selector.strip_prefix("uuid:") {
        return elem.attributes.get("U").map(|u| u == uuid).unwrap_or(false);
    }
    if let Some(gid) = selector.strip_prefix("gid:") {
        return elem
            .attributes
            .get("gid")
            .map(|g| g.eq_ignore_ascii_case(gid))
            .unwrap_or(false);
    }
    if let Some(type_name) = selector.strip_prefix("Type:") {
        return elem
            .attributes
            .get("Type")
            .map(|t| t.eq_ignore_ascii_case(type_name))
            .unwrap_or(false);
    }

    // Default: match by Title (case-insensitive contains)
    elem.attributes
        .get("Title")
        .map(|t| t.to_lowercase().contains(&selector.to_lowercase()))
        .unwrap_or(false)
}

// ── Data types ───────────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize)]
pub struct PropertyValue {
    pub value: String,
    pub type_code: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ConnectorInfo {
    pub kind: String,
    pub target: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ElementDescription {
    pub element_type: String,
    pub title: String,
    pub uuid: String,
    pub gid: String,
    pub room_uuid: String,
    pub category_uuid: String,
    pub properties: HashMap<String, PropertyValue>,
    pub connectors: Vec<ConnectorInfo>,
    pub children: Vec<String>,
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_XML: &[u8] = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="Place" V="175" U="room-1" Title="Kitchen" WF="16384"/>
  <C Type="Place" V="175" U="room-2" Title="Zentral" WF="16384"/>
  <C Type="Category" V="175" U="cat-1" Title="Wetter"/>
  <C Type="Plugin" gid="Mqtt" U="mqtt-1" Title="MQTT">
    <SET>
      <mqtt_broker_address t="11" v="192.168.1.1"/>
      <mqtt_broker_port t="7" v="1883"/>
    </SET>
    <C Type="GenTSensor" U="sensor-1" Title="Temp Sub">
      <IoData Cr="cat-1" Pr="room-1"/>
    </C>
  </C>
  <C Type="WeatherData" U="wd-1" Title="Temperatur">
    <IoData Cr="cat-1" Pr="room-1"/>
  </C>
  <C Type="WeatherData" U="wd-2" Title="Wind">
    <IoData Cr="cat-1" Pr="room-1"/>
  </C>
  <C Type="SysVar" U="sv-1" Title="Aussentemp">
    <IoData Cr="cat-1" Pr="room-1"/>
  </C>
</ControlList>"#;

    #[test]
    fn test_load_and_write_preserves_content() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let output = editor.to_bytes().unwrap();
        // Verify it's valid XML
        let _ = ConfigEditor::load(&output).unwrap();
    }

    #[test]
    fn test_bom_preservation() {
        let mut with_bom = Vec::new();
        with_bom.extend_from_slice(UTF8_BOM);
        with_bom.extend_from_slice(SAMPLE_XML);
        let editor = ConfigEditor::load(&with_bom).unwrap();
        assert!(editor.had_bom);
        let output = editor.to_bytes().unwrap();
        assert!(output.starts_with(UTF8_BOM));
    }

    #[test]
    fn test_find_by_title() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let matches = editor.find_elements("Temperatur");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_find_by_gid() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let matches = editor.find_elements("gid:Mqtt");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_find_by_type() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let matches = editor.find_elements("Type:WeatherData");
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_find_by_uuid() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let matches = editor.find_elements("uuid:wd-1");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_set_property_update() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let msg = editor
            .set_property("gid:Mqtt", "mqtt_broker_address", "10.0.0.1", "11")
            .unwrap();
        assert!(msg.contains("10.0.0.1"));

        let desc = editor.describe("gid:Mqtt").unwrap();
        assert_eq!(desc.properties["mqtt_broker_address"].value, "10.0.0.1");
    }

    #[test]
    fn test_set_property_create() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        editor
            .set_property("gid:Mqtt", "mqtt_auth_pwd", "secret", "11")
            .unwrap();

        let desc = editor.describe("gid:Mqtt").unwrap();
        assert_eq!(desc.properties["mqtt_auth_pwd"].value, "secret");
    }

    #[test]
    fn test_set_attribute() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        editor
            .set_attribute("uuid:wd-1", "Title", "Neue Temperatur")
            .unwrap();

        let matches = editor.find_elements("Neue Temperatur");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_move_to_room() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let (count, uuid) = editor.move_to_room("WeatherData", "Zentral", &[]).unwrap();
        assert_eq!(count, 2);
        assert_eq!(uuid, "room-2");

        // Verify IoData was updated
        let output = editor.to_bytes().unwrap();
        let check = ConfigEditor::load(&output).unwrap();
        let desc = check.describe("uuid:wd-1").unwrap();
        assert_eq!(desc.room_uuid, "room-2");
    }

    #[test]
    fn test_describe() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let desc = editor.describe("gid:Mqtt").unwrap();
        assert_eq!(desc.element_type, "Plugin");
        assert_eq!(desc.title, "MQTT");
        assert!(!desc.properties.is_empty());
        assert_eq!(desc.properties["mqtt_broker_address"].value, "192.168.1.1");
        assert_eq!(desc.children.len(), 1);
    }

    #[test]
    fn test_add_room() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let uuid = editor.add_room("Garten").unwrap();
        assert!(!uuid.is_empty());

        let matches = editor.find_elements("Garten");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_add_room_duplicate_fails() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let result = editor.add_room("Kitchen");
        assert!(result.is_err());
    }

    #[test]
    fn test_roundtrip_write_read() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        editor.set_property("gid:Mqtt", "mqtt_broker_address", "10.0.0.1", "11").unwrap();
        editor.set_attribute("uuid:wd-1", "Title", "NewTemp").unwrap();
        let (count, _) = editor.move_to_room("WeatherData", "Zentral", &[]).unwrap();
        assert_eq!(count, 2);

        let output = editor.to_bytes().unwrap();
        let check = ConfigEditor::load(&output).unwrap();

        let desc = check.describe("gid:Mqtt").unwrap();
        assert_eq!(desc.properties["mqtt_broker_address"].value, "10.0.0.1");

        let matches = check.find_elements("NewTemp");
        assert_eq!(matches.len(), 1);
    }
}
