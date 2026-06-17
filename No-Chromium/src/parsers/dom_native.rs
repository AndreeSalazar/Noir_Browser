// AUTO-GENERATED NATIVE DOM
// Based on official WebIDL ADN
#![allow(non_snake_case, dead_code)]

pub struct Event {
    pub id: u64,
}

impl Event {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
    pub fn get_type(&self) -> &str {
        "type"
    }
    pub fn get_eventPhase(&self) -> &str {
        "eventPhase"
    }
    pub fn get_cancelBubble(&self) -> &str {
        "cancelBubble"
    }
    pub fn get_bubbles(&self) -> &str {
        "bubbles"
    }
    pub fn get_cancelable(&self) -> &str {
        "cancelable"
    }
    pub fn get_returnValue(&self) -> &str {
        "returnValue"
    }
    pub fn get_defaultPrevented(&self) -> &str {
        "defaultPrevented"
    }
    pub fn get_composed(&self) -> &str {
        "composed"
    }
    pub fn get_isTrusted(&self) -> &str {
        "isTrusted"
    }
    pub fn get_timeStamp(&self) -> &str {
        "timeStamp"
    }
}

/// Window Interface (Inherits: Base)
pub struct Window {
    pub id: u64,
}

impl Window {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}

/// CustomEvent Interface (Inherits: Event)
pub struct CustomEvent {
    pub id: u64,
    pub super_class: Box<Event>,
}

impl CustomEvent {
    pub fn new(id: u64) -> Self {
        Self { id, super_class: Box::new(Event::new(id)) }
    }
    pub fn get_detail(&self) -> &str {
        "detail"
    }
}

/// EventTarget Interface (Inherits: Base)
pub struct EventTarget {
    pub id: u64,
}

impl EventTarget {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}

/// EventListener Interface (Inherits: Base)
pub struct EventListener {
    pub id: u64,
}

impl EventListener {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}

/// AbortController Interface (Inherits: Base)
pub struct AbortController {
    pub id: u64,
}

impl AbortController {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
    pub fn get_signal(&self) -> &str {
        "signal"
    }
}

/// AbortSignal Interface (Inherits: EventTarget)
pub struct AbortSignal {
    pub id: u64,
    pub super_class: Box<EventTarget>,
}

impl AbortSignal {
    pub fn new(id: u64) -> Self {
        Self { id, super_class: Box::new(EventTarget::new(id)) }
    }
    pub fn get_aborted(&self) -> &str {
        "aborted"
    }
    pub fn get_reason(&self) -> &str {
        "reason"
    }
    pub fn get_onabort(&self) -> &str {
        "onabort"
    }
}

/// NodeList Interface (Inherits: Base)
pub struct NodeList {
    pub id: u64,
}

impl NodeList {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
    pub fn get_length(&self) -> &str {
        "length"
    }
}

/// HTMLCollection Interface (Inherits: Base)
pub struct HTMLCollection {
    pub id: u64,
}

impl HTMLCollection {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
    pub fn get_length(&self) -> &str {
        "length"
    }
}

/// MutationObserver Interface (Inherits: Base)
pub struct MutationObserver {
    pub id: u64,
}

impl MutationObserver {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}

/// MutationRecord Interface (Inherits: Base)
pub struct MutationRecord {
    pub id: u64,
}

impl MutationRecord {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
    pub fn get_type(&self) -> &str {
        "type"
    }
    pub fn get_target(&self) -> &str {
        "target"
    }
    pub fn get_addedNodes(&self) -> &str {
        "addedNodes"
    }
    pub fn get_removedNodes(&self) -> &str {
        "removedNodes"
    }
}

/// Node Interface (Inherits: EventTarget)
pub struct Node {
    pub id: u64,
    pub super_class: Box<EventTarget>,
}

impl Node {
    pub fn new(id: u64) -> Self {
        Self { id, super_class: Box::new(EventTarget::new(id)) }
    }
    pub fn get_nodeType(&self) -> &str {
        "nodeType"
    }
    pub fn get_nodeName(&self) -> &str {
        "nodeName"
    }
    pub fn get_baseURI(&self) -> &str {
        "baseURI"
    }
    pub fn get_isConnected(&self) -> &str {
        "isConnected"
    }
    pub fn get_childNodes(&self) -> &str {
        "childNodes"
    }
}

/// DOMImplementation Interface (Inherits: Base)
pub struct DOMImplementation {
    pub id: u64,
}

impl DOMImplementation {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}

/// DocumentType Interface (Inherits: Node)
pub struct DocumentType {
    pub id: u64,
    pub super_class: Box<Node>,
}

impl DocumentType {
    pub fn new(id: u64) -> Self {
        Self { id, super_class: Box::new(Node::new(id)) }
    }
    pub fn get_name(&self) -> &str {
        "name"
    }
    pub fn get_publicId(&self) -> &str {
        "publicId"
    }
    pub fn get_systemId(&self) -> &str {
        "systemId"
    }
}

/// DocumentFragment Interface (Inherits: Node)
pub struct DocumentFragment {
    pub id: u64,
    pub super_class: Box<Node>,
}

impl DocumentFragment {
    pub fn new(id: u64) -> Self {
        Self { id, super_class: Box::new(Node::new(id)) }
    }
}

/// ShadowRoot Interface (Inherits: DocumentFragment)
pub struct ShadowRoot {
    pub id: u64,
    pub super_class: Box<DocumentFragment>,
}

impl ShadowRoot {
    pub fn new(id: u64) -> Self {
        Self { id, super_class: Box::new(DocumentFragment::new(id)) }
    }
    pub fn get_mode(&self) -> &str {
        "mode"
    }
    pub fn get_delegatesFocus(&self) -> &str {
        "delegatesFocus"
    }
    pub fn get_slotAssignment(&self) -> &str {
        "slotAssignment"
    }
    pub fn get_clonable(&self) -> &str {
        "clonable"
    }
    pub fn get_serializable(&self) -> &str {
        "serializable"
    }
    pub fn get_host(&self) -> &str {
        "host"
    }
    pub fn get_onslotchange(&self) -> &str {
        "onslotchange"
    }
}

/// Element Interface (Inherits: Node)
pub struct Element {
    pub id: u64,
    pub super_class: Box<Node>,
}

impl Element {
    pub fn new(id: u64) -> Self {
        Self { id, super_class: Box::new(Node::new(id)) }
    }
    pub fn get_localName(&self) -> &str {
        "localName"
    }
    pub fn get_tagName(&self) -> &str {
        "tagName"
    }
    pub fn get_id(&self) -> &str {
        "id"
    }
    pub fn get_className(&self) -> &str {
        "className"
    }
    pub fn get_classList(&self) -> &str {
        "classList"
    }
    pub fn get_slot(&self) -> &str {
        "slot"
    }
    pub fn get_attributes(&self) -> &str {
        "attributes"
    }
}

/// NamedNodeMap Interface (Inherits: Base)
pub struct NamedNodeMap {
    pub id: u64,
}

impl NamedNodeMap {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
    pub fn get_length(&self) -> &str {
        "length"
    }
}

/// Attr Interface (Inherits: Node)
pub struct Attr {
    pub id: u64,
    pub super_class: Box<Node>,
}

impl Attr {
    pub fn new(id: u64) -> Self {
        Self { id, super_class: Box::new(Node::new(id)) }
    }
    pub fn get_localName(&self) -> &str {
        "localName"
    }
    pub fn get_name(&self) -> &str {
        "name"
    }
    pub fn get_value(&self) -> &str {
        "value"
    }
    pub fn get_specified(&self) -> &str {
        "specified"
    }
}

/// CharacterData Interface (Inherits: Node)
pub struct CharacterData {
    pub id: u64,
    pub super_class: Box<Node>,
}

impl CharacterData {
    pub fn new(id: u64) -> Self {
        Self { id, super_class: Box::new(Node::new(id)) }
    }
    pub fn get_length(&self) -> &str {
        "length"
    }
}

/// Text Interface (Inherits: CharacterData)
pub struct Text {
    pub id: u64,
    pub super_class: Box<CharacterData>,
}

impl Text {
    pub fn new(id: u64) -> Self {
        Self { id, super_class: Box::new(CharacterData::new(id)) }
    }
    pub fn get_wholeText(&self) -> &str {
        "wholeText"
    }
}

/// CDATASection Interface (Inherits: Text)
pub struct CDATASection {
    pub id: u64,
    pub super_class: Box<Text>,
}

impl CDATASection {
    pub fn new(id: u64) -> Self {
        Self { id, super_class: Box::new(Text::new(id)) }
    }
}

/// ProcessingInstruction Interface (Inherits: CharacterData)
pub struct ProcessingInstruction {
    pub id: u64,
    pub super_class: Box<CharacterData>,
}

impl ProcessingInstruction {
    pub fn new(id: u64) -> Self {
        Self { id, super_class: Box::new(CharacterData::new(id)) }
    }
    pub fn get_target(&self) -> &str {
        "target"
    }
}

/// Comment Interface (Inherits: CharacterData)
pub struct Comment {
    pub id: u64,
    pub super_class: Box<CharacterData>,
}

impl Comment {
    pub fn new(id: u64) -> Self {
        Self { id, super_class: Box::new(CharacterData::new(id)) }
    }
}

/// AbstractRange Interface (Inherits: Base)
pub struct AbstractRange {
    pub id: u64,
}

impl AbstractRange {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
    pub fn get_startContainer(&self) -> &str {
        "startContainer"
    }
    pub fn get_startOffset(&self) -> &str {
        "startOffset"
    }
    pub fn get_endContainer(&self) -> &str {
        "endContainer"
    }
    pub fn get_endOffset(&self) -> &str {
        "endOffset"
    }
    pub fn get_collapsed(&self) -> &str {
        "collapsed"
    }
}

/// StaticRange Interface (Inherits: AbstractRange)
pub struct StaticRange {
    pub id: u64,
    pub super_class: Box<AbstractRange>,
}

impl StaticRange {
    pub fn new(id: u64) -> Self {
        Self { id, super_class: Box::new(AbstractRange::new(id)) }
    }
}

/// Range Interface (Inherits: AbstractRange)
pub struct Range {
    pub id: u64,
    pub super_class: Box<AbstractRange>,
}

impl Range {
    pub fn new(id: u64) -> Self {
        Self { id, super_class: Box::new(AbstractRange::new(id)) }
    }
    pub fn get_commonAncestorContainer(&self) -> &str {
        "commonAncestorContainer"
    }
}

/// NodeIterator Interface (Inherits: Base)
pub struct NodeIterator {
    pub id: u64,
}

impl NodeIterator {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
    pub fn get_root(&self) -> &str {
        "root"
    }
    pub fn get_referenceNode(&self) -> &str {
        "referenceNode"
    }
    pub fn get_pointerBeforeReferenceNode(&self) -> &str {
        "pointerBeforeReferenceNode"
    }
    pub fn get_whatToShow(&self) -> &str {
        "whatToShow"
    }
}

/// TreeWalker Interface (Inherits: Base)
pub struct TreeWalker {
    pub id: u64,
}

impl TreeWalker {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
    pub fn get_root(&self) -> &str {
        "root"
    }
    pub fn get_whatToShow(&self) -> &str {
        "whatToShow"
    }
    pub fn get_currentNode(&self) -> &str {
        "currentNode"
    }
}

/// NodeFilter Interface (Inherits: Base)
pub struct NodeFilter {
    pub id: u64,
}

impl NodeFilter {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}

/// DOMTokenList Interface (Inherits: Base)
pub struct DOMTokenList {
    pub id: u64,
}

impl DOMTokenList {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
    pub fn get_length(&self) -> &str {
        "length"
    }
    pub fn get_value(&self) -> &str {
        "value"
    }
}

/// XPathResult Interface (Inherits: Base)
pub struct XPathResult {
    pub id: u64,
}

impl XPathResult {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
    pub fn get_resultType(&self) -> &str {
        "resultType"
    }
    pub fn get_numberValue(&self) -> &str {
        "numberValue"
    }
    pub fn get_stringValue(&self) -> &str {
        "stringValue"
    }
    pub fn get_booleanValue(&self) -> &str {
        "booleanValue"
    }
    pub fn get_invalidIteratorState(&self) -> &str {
        "invalidIteratorState"
    }
    pub fn get_snapshotLength(&self) -> &str {
        "snapshotLength"
    }
}

/// XPathExpression Interface (Inherits: Base)
pub struct XPathExpression {
    pub id: u64,
}

impl XPathExpression {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}

/// XPathNSResolver Interface (Inherits: Base)
pub struct XPathNSResolver {
    pub id: u64,
}

impl XPathNSResolver {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}

/// XPathEvaluator Interface (Inherits: Base)
pub struct XPathEvaluator {
    pub id: u64,
}

impl XPathEvaluator {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}

/// XSLTProcessor Interface (Inherits: Base)
pub struct XSLTProcessor {
    pub id: u64,
}

impl XSLTProcessor {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}

/// Document Interface (Inherits: Node)
pub struct Document {
    pub id: u64,
    pub super_class: Box<Node>,
}

impl Document {
    pub fn new(id: u64) -> Self {
        Self { id, super_class: Box::new(Node::new(id)) }
    }
    pub fn get_URL(&self) -> &str {
        "URL"
    }
}
