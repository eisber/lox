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

    /// Add a child element under a parent. Returns the generated UUID.
    pub fn add_element(
        &mut self,
        parent_selector: &str,
        element_type: &str,
        title: &str,
        gid: Option<&str>,
        room_uuid: Option<&str>,
        category_uuid: Option<&str>,
        properties: &[(&str, &str, &str)], // (name, value, type_code)
    ) -> Result<String> {
        let parent_path = self.require_one(parent_selector)?;
        let uuid = format!(
            "{:08x}-{:04x}-{:04x}-ffff000000000000",
            rand::random::<u32>(),
            rand::random::<u16>(),
            rand::random::<u16>()
        );

        let mut elem = Element::new("C");
        elem.attributes
            .insert("Type".to_string(), element_type.to_string());
        elem.attributes
            .insert("V".to_string(), "175".to_string());
        elem.attributes.insert("U".to_string(), uuid.clone());
        elem.attributes
            .insert("Title".to_string(), title.to_string());
        if let Some(g) = gid {
            elem.attributes.insert("gid".to_string(), g.to_string());
        }

        // Add IoData if room or category specified
        if room_uuid.is_some() || category_uuid.is_some() {
            let mut iodata = Element::new("IoData");
            if let Some(r) = room_uuid {
                iodata.attributes.insert("Pr".to_string(), r.to_string());
            }
            if let Some(c) = category_uuid {
                iodata.attributes.insert("Cr".to_string(), c.to_string());
            }
            elem.children
                .push(xmltree::XMLNode::Element(iodata));
        }

        // Add properties
        if !properties.is_empty() {
            let mut set = Element::new("SET");
            for (name, value, type_code) in properties {
                let mut prop = Element::new(*name);
                prop.attributes
                    .insert("t".to_string(), type_code.to_string());
                prop.attributes
                    .insert("v".to_string(), value.to_string());
                set.children
                    .push(xmltree::XMLNode::Element(prop));
            }
            elem.children
                .push(xmltree::XMLNode::Element(set));
        }

        let parent = self.get_element_mut(&parent_path);
        parent
            .children
            .push(xmltree::XMLNode::Element(elem));

        Ok(uuid)
    }

    /// Remove an element by UUID.
    pub fn remove_element(&mut self, uuid: &str) -> Result<String> {
        remove_by_uuid(&mut self.root.children, uuid)
    }

    /// Wire two connectors: set source_element.connector_name → target_element's connector UUID.
    ///
    /// `source`: element selector (e.g. "Kitchen Light")
    /// `source_connector`: connector name on the source (e.g. "On", "AQ1")
    /// `target`: element selector for the target
    /// `target_connector`: connector name on the target (e.g. "I", "Q")
    pub fn wire(
        &mut self,
        source: &str,
        source_connector: &str,
        target: &str,
        target_connector: &str,
    ) -> Result<String> {
        // Find target element and its connector UUID
        let target_path = self.require_one(target)?;
        let target_elem = self.get_element(&target_path);
        let target_title = target_elem.attributes.get("Title").cloned().unwrap_or_default();

        let target_co_uuid = target_elem
            .children
            .iter()
            .find_map(|c| {
                c.as_element().and_then(|e| {
                    if e.name == "Co" && e.attributes.get("K").map(|k| k == target_connector).unwrap_or(false) {
                        e.attributes.get("U").cloned()
                    } else {
                        None
                    }
                })
            })
            .ok_or_else(|| {
                let available: Vec<String> = target_elem
                    .children
                    .iter()
                    .filter_map(|c| c.as_element())
                    .filter(|e| e.name == "Co")
                    .filter_map(|e| e.attributes.get("K").cloned())
                    .collect();
                crate::errors::not_found_error(
                    "Connector",
                    target_connector,
                    &available,
                    &format!("lox config control describe <file> \"{}\"", target),
                )
            })?;

        // Find source element and update its connector
        let source_path = self.require_one(source)?;
        let source_elem = self.get_element_mut(&source_path);
        let source_title = source_elem.attributes.get("Title").cloned().unwrap_or_default();

        let source_co = source_elem
            .children
            .iter_mut()
            .find_map(|c| {
                c.as_mut_element().and_then(|e| {
                    if e.name == "Co" && e.attributes.get("K").map(|k| k == source_connector).unwrap_or(false) {
                        Some(e)
                    } else {
                        None
                    }
                })
            })
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Connector '{}' not found on '{}'",
                    source_connector,
                    source_title
                )
            })?;

        let old_target = source_co.attributes.get("U").cloned().unwrap_or_default();
        source_co
            .attributes
            .insert("U".to_string(), target_co_uuid.clone());

        Ok(format!(
            "Wired {}.{} → {}.{} ({})",
            source_title, source_connector, target_title, target_connector, target_co_uuid
        ))
    }

    /// Disconnect a connector (set its target UUID to empty/zero).
    pub fn unwire(&mut self, selector: &str, connector_name: &str) -> Result<String> {
        let path = self.require_one(selector)?;
        let elem = self.get_element_mut(&path);
        let title = elem.attributes.get("Title").cloned().unwrap_or_default();

        let co = elem
            .children
            .iter_mut()
            .find_map(|c| {
                c.as_mut_element().and_then(|e| {
                    if e.name == "Co" && e.attributes.get("K").map(|k| k == connector_name).unwrap_or(false) {
                        Some(e)
                    } else {
                        None
                    }
                })
            })
            .ok_or_else(|| anyhow::anyhow!("Connector '{}' not found on '{}'", connector_name, title))?;

        let old = co.attributes.get("U").cloned().unwrap_or_default();
        co.attributes.remove("U");

        Ok(format!("Unwired {}.{} (was {})", title, connector_name, old))
    }

    /// List all connectors and their wiring for an element.
    pub fn list_wires(&self, selector: &str) -> Result<Vec<WireInfo>> {
        let path = self.require_one(selector)?;
        let elem = self.get_element(&path);

        let mut wires = Vec::new();
        for child in &elem.children {
            if let Some(co) = child.as_element() {
                if co.name == "Co" {
                    let name = co.attributes.get("K").cloned().unwrap_or_default();
                    let target_uuid = co.attributes.get("U").cloned().unwrap_or_default();

                    // Classify direction
                    let direction = if name.starts_with('I') || name.starts_with("AI") || name == "Input" {
                        "input"
                    } else if name.starts_with('Q') || name.starts_with("AQ") || name.starts_with("Output") {
                        "output"
                    } else {
                        "parameter"
                    };

                    let connected = !target_uuid.is_empty()
                        && target_uuid != "00000000-0000-0000-0000000000000000";

                    wires.push(WireInfo {
                        connector: name,
                        direction: direction.to_string(),
                        target_uuid,
                        connected,
                    });
                }
            }
        }
        Ok(wires)
    }

    /// List all MQTT topics (GenTSensor subscriptions + GenTActor publishes).
    pub fn list_mqtt_topics(&self) -> Vec<MqttTopic> {
        let mut topics = Vec::new();
        self.collect_mqtt_topics(&self.root, &mut topics);
        topics
    }

    fn collect_mqtt_topics(&self, elem: &Element, topics: &mut Vec<MqttTopic>) {
        if elem.name == "C" {
            if let Some(t) = elem.attributes.get("Type") {
                if t == "GenTSensor" || t == "GenTActor" {
                    let title = elem
                        .attributes
                        .get("Title")
                        .cloned()
                        .unwrap_or_default();
                    let direction = if t == "GenTSensor" {
                        "subscribe"
                    } else {
                        "publish"
                    };

                    // Get topic from SET properties
                    let mut topic = String::new();
                    let mut qos = String::new();
                    for child in &elem.children {
                        if let Some(set) = child.as_element() {
                            if set.name == "SET" {
                                for prop in &set.children {
                                    if let Some(p) = prop.as_element() {
                                        if p.name == "mqtt_topic" {
                                            topic = p
                                                .attributes
                                                .get("v")
                                                .cloned()
                                                .unwrap_or_default();
                                        }
                                        if p.name == "mqtt_qos" {
                                            qos = p
                                                .attributes
                                                .get("v")
                                                .cloned()
                                                .unwrap_or_default();
                                        }
                                    }
                                }
                            }
                        }
                    }

                    topics.push(MqttTopic {
                        title,
                        direction: direction.to_string(),
                        topic,
                        qos,
                    });
                }
            }
        }
        for child in &elem.children {
            if let Some(child_elem) = child.as_element() {
                self.collect_mqtt_topics(child_elem, topics);
            }
        }
    }
}

/// Remove an element by UUID from a children list (recursive standalone function).
fn remove_by_uuid(children: &mut Vec<xmltree::XMLNode>, uuid: &str) -> Result<String> {
    for i in 0..children.len() {
        if let Some(elem) = children[i].as_element() {
            if elem.attributes.get("U").map(|u| u == uuid).unwrap_or(false) {
                let title = elem
                    .attributes
                    .get("Title")
                    .cloned()
                    .unwrap_or_default();
                children.remove(i);
                return Ok(title);
            }
        }
    }
    for child in children.iter_mut() {
        if let Some(elem) = child.as_mut_element() {
            if let Ok(title) = remove_by_uuid(&mut elem.children, uuid) {
                return Ok(title);
            }
        }
    }
    bail!("Element with UUID '{}' not found", uuid)
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

#[derive(Debug, serde::Serialize)]
pub struct WireInfo {
    pub connector: String,
    pub direction: String,
    pub target_uuid: String,
    pub connected: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct MqttTopic {
    pub title: String,
    pub direction: String,
    pub topic: String,
    pub qos: String,
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
      <Co K="Text" U="co-sensor1-text"/>
      <IoData Cr="cat-1" Pr="room-1"/>
    </C>
  </C>
  <C Type="WeatherData" U="wd-1" Title="Temperatur">
    <Co K="AQ" U="co-wd1-aq"/>
    <IoData Cr="cat-1" Pr="room-1"/>
  </C>
  <C Type="WeatherData" U="wd-2" Title="Wind">
    <Co K="AQ" U="co-wd2-aq"/>
    <IoData Cr="cat-1" Pr="room-1"/>
  </C>
  <C Type="SysVar" U="sv-1" Title="Aussentemp">
    <Co K="AQ" U="co-sv1-aq"/>
    <Co K="AI" U="co-sv1-ai"/>
    <Co K="Q" U="co-sv1-q"/>
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

    #[test]
    fn test_add_element() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let uuid = editor
            .add_element(
                "gid:Mqtt",
                "GenTSensor",
                "New Sensor",
                Some("Mqtt.subt"),
                Some("room-2"),
                Some("cat-1"),
                &[("mqtt_topic", "test/topic", "11")],
            )
            .unwrap();
        assert!(!uuid.is_empty());

        let desc = editor.describe(&format!("uuid:{}", uuid)).unwrap();
        assert_eq!(desc.element_type, "GenTSensor");
        assert_eq!(desc.title, "New Sensor");
        assert_eq!(desc.properties["mqtt_topic"].value, "test/topic");
    }

    #[test]
    fn test_remove_element() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        assert_eq!(editor.find_elements("uuid:wd-2").len(), 1);
        let title = editor.remove_element("wd-2").unwrap();
        assert_eq!(title, "Wind");
        assert_eq!(editor.find_elements("uuid:wd-2").len(), 0);
    }

    #[test]
    fn test_remove_nonexistent_fails() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let result = editor.remove_element("nonexistent-uuid");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_mqtt_topics() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        // The sample has one GenTSensor but no mqtt_topic property set
        let topics = editor.list_mqtt_topics();
        assert_eq!(topics.len(), 1);
        assert_eq!(topics[0].direction, "subscribe");
        assert_eq!(topics[0].title, "Temp Sub");

        // Add one with a topic
        editor
            .add_element(
                "gid:Mqtt",
                "GenTActor",
                "Publisher",
                Some("Mqtt.pubt"),
                None,
                None,
                &[("mqtt_topic", "home/status", "11")],
            )
            .unwrap();
        let topics = editor.list_mqtt_topics();
        assert_eq!(topics.len(), 2);
        let pub_topic = topics.iter().find(|t| t.direction == "publish").unwrap();
        assert_eq!(pub_topic.topic, "home/status");
    }

    #[test]
    fn test_list_wires() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        // WeatherData has 1 connector (AQ)
        let wires = editor.list_wires("uuid:wd-1").unwrap();
        assert_eq!(wires.len(), 1);
        assert_eq!(wires[0].connector, "AQ");
    }

    #[test]
    fn test_wire_and_unwire() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();

        // Wire WeatherData.AQ → SysVar (but SysVar has no Co in sample, so wire to sensor)
        // Wire sensor.Text → WeatherData.AQ
        let msg = editor.wire("uuid:sensor-1", "Text", "uuid:wd-1", "AQ").unwrap();
        assert!(msg.contains("Wired"));

        // Verify it's connected
        let wires = editor.list_wires("uuid:sensor-1").unwrap();
        let text_co = wires.iter().find(|w| w.connector == "Text").unwrap();
        assert!(text_co.connected);

        // Unwire
        let msg = editor.unwire("uuid:sensor-1", "Text").unwrap();
        assert!(msg.contains("Unwired"));

        // Verify disconnected
        let wires = editor.list_wires("uuid:sensor-1").unwrap();
        let text_co = wires.iter().find(|w| w.connector == "Text").unwrap();
        assert!(!text_co.connected);
    }
}
