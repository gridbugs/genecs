extern crate tomson;
extern crate handlebars;
extern crate rustc_serialize;

use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::fs::File;
use std::mem;

use tomson::Toml;
use handlebars::Handlebars;
use rustc_serialize::json::{self, Json};


const TEMPLATE: &'static str = r#"// Automatically generated. Do not edit.
#![allow(unused_imports)]

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::cell::{UnsafeCell, RefCell, Ref, RefMut};
use std::slice;
use std::usize;

{{#each imports}}
use {{ this }};
{{/each}}

pub type EntityId = u64;

pub type EntityMap<T> = BTreeMap<EntityId, T>;
pub type EntitySet = BTreeSet<EntityId>;

pub const NUM_COMPONENTS: usize = {{num_components}};

const WORD_SIZE: usize = {{word_size}};
const WORD_BITS: usize = {{word_bits}};

const COMPONENT_TYPE_SET_NUM_WORDS: usize = {{component_set_num_words}};

pub type ComponentType = usize;

pub mod component_type {
    use std::usize;

{{#each component}}
    pub const {{id_uppercase}}: usize = {{index}};
{{/each}}
    pub const INVALID_COMPONENT: usize = usize::MAX;
}

pub struct ComponentTypeSet {
    bitfields: [usize; COMPONENT_TYPE_SET_NUM_WORDS],
}

pub struct ComponentTypeSetIter {
    bitfields: [usize; COMPONENT_TYPE_SET_NUM_WORDS],
    index: usize,
}

impl ComponentTypeSetIter {
    fn new(bitfields: [usize; COMPONENT_TYPE_SET_NUM_WORDS]) -> Self {
        ComponentTypeSetIter {
            bitfields: bitfields,
            index: 0,
        }
    }
}

impl Iterator for ComponentTypeSetIter {
    type Item = ComponentType;
    fn next(&mut self) -> Option<Self::Item> {
        while self.index < COMPONENT_TYPE_SET_NUM_WORDS && self.bitfields[self.index] == 0 {
            self.index += 1;
        }
        if self.index == COMPONENT_TYPE_SET_NUM_WORDS {
            return None;
        }

        let trailing = self.bitfields[self.index].trailing_zeros() as usize;
        self.bitfields[self.index] &= !(1 << trailing);
        Some(self.index * WORD_BITS + trailing)
    }
}

impl ComponentTypeSet {
    pub fn new() -> Self {
        ComponentTypeSet {
            bitfields: [0; COMPONENT_TYPE_SET_NUM_WORDS],
        }
    }

    pub fn is_empty(&self) -> bool {
        for b in &self.bitfields {
            if *b != 0 {
                return false;
            }
        }

        true
    }

    pub fn clear(&mut self) {
        for b in &mut self.bitfields {
            *b = 0;
        }
    }

    pub fn iter(&self) -> ComponentTypeSetIter {
        ComponentTypeSetIter::new(self.bitfields)
    }

{{#each component}}
    pub fn contains_{{id}}(&self) -> bool {
        self.bitfields[{{set_index}}] & (1 << {{set_bit}}) != 0
    }

    pub fn insert_{{id}}(&mut self) {
        self.bitfields[{{set_index}}] |= 1 << {{set_bit}};
    }

    pub fn remove_{{id}}(&mut self) {
        self.bitfields[{{set_index}}] &= !(1 << {{set_bit}});
    }
{{/each}}
}

struct ComponentDirtyFlags {
    insert: bool,
    remove: bool,
}

impl ComponentDirtyFlags {
    fn new() -> Self {
        ComponentDirtyFlags {
            insert: false,
            remove: false,
        }
    }

    fn clean(&mut self) {
        self.insert = false;
        self.remove = false;
    }
}

struct DirtyComponentTracker {
{{#each queried_components}}
    {{ @key }}: ComponentDirtyFlags,
{{/each}}
}

impl DirtyComponentTracker {
    fn new() -> Self {
        DirtyComponentTracker {
{{#each queried_components}}
            {{ @key }}: ComponentDirtyFlags::new(),
{{/each}}
        }
    }

{{#each query}}
    fn should_populate_{{id}}(&self) -> bool {
        (true {{#each components}}&& self.{{id}}.insert {{/each}}) ||
        (false {{#each components}}|| self.{{id}}.remove {{/each}})
    }
{{/each}}
}

pub struct EcsTable {
{{#each component}}
    {{#if type}}
        {{#if container}}
    {{id}}: EntityMap<{{container}}<{{type}}>>,
        {{else}}
    {{id}}: EntityMap<{{type}}>,
        {{/if}}
    {{else}}
    {{id}}: EntitySet,
    {{/if}}
{{/each}}
}

impl EcsTable {
    pub fn new() -> Self {
        EcsTable {
{{#each component}}
            {{id}}: {{#if type}} EntityMap::new() {{else}} EntitySet::new() {{/if}},
{{/each}}
        }
    }

{{#each component}}
    {{#if type}}
        {{#if container}}
    pub fn insert_{{id}}(&mut self, entity: EntityId, value: {{type}}) {
        self.{{id}}.insert(entity, {{container}}::new(value));
    }
        {{else}}
    pub fn insert_{{id}}(&mut self, entity: EntityId, value: {{type}}) {
        self.{{id}}.insert(entity, value);
    }
        {{/if}}

    pub fn contains_{{id}}(&self, entity: EntityId) -> bool {
        self.{{id}}.contains_key(&entity)
    }

        {{#if copy}}
    pub fn {{id}}(&self, entity: EntityId) -> Option<{{type}}> {
        self.{{id}}.get(&entity).map(|r| *r)
    }
    pub fn {{id}}_ref(&self, entity: EntityId) -> Option<&{{type}}> {
        self.{{id}}.get(&entity)
    }
        {{else}}
            {{#if container}}

    pub fn {{id}}(&self, entity: EntityId) -> Option<&{{container}}<{{type}}>> {
        self.{{id}}.get(&entity)
    }
                {{#if RefCell}}
    pub fn {{id}}_borrow(&self, entity: EntityId) -> Option<Ref<{{type}}>> {
        self.{{id}}.get(&entity).map(|e| e.borrow())
    }
    pub fn {{id}}_borrow_mut(&self, entity: EntityId) -> Option<RefMut<{{type}}>> {
        self.{{id}}.get(&entity).map(|e| e.borrow_mut())
    }
                {{/if}}
                {{#if UnsafeCell}}
    pub fn {{id}}_unsafe_get_mut(&self, entity: EntityId) -> Option<&mut {{type}}> {
        unsafe {
            self.{{id}}.get(&entity).map(|e| &mut *e.get())
        }
    }
    pub fn {{id}}_unsafe_get(&self, entity: EntityId) -> Option<&{{type}}> {
        unsafe {
            self.{{id}}.get(&entity).map(|e| &*e.get())
        }
    }
                {{/if}}

            {{else}}
    pub fn {{id}}(&self, entity: EntityId) -> Option<&{{type}}> {
        self.{{id}}.get(&entity)
    }
            {{/if}}
        {{/if}}


        {{#if container}}
    pub fn {{id}}_mut(&mut self, entity: EntityId) -> Option<&mut {{container}}<{{type}}>> {
        self.{{id}}.get_mut(&entity)
    }
        {{else}}
    pub fn {{id}}_mut(&mut self, entity: EntityId) -> Option<&mut {{type}}> {
        self.{{id}}.get_mut(&entity)
    }
        {{/if}}
    {{else}}
    pub fn insert_{{id}}(&mut self, entity: EntityId) {
        self.{{id}}.insert(entity);
    }

    pub fn contains_{{id}}(&self, entity: EntityId) -> bool {
        self.{{id}}.contains(&entity)
    }
    {{/if}}

    pub fn remove_{{id}}(&mut self, entity: EntityId) {
        self.{{id}}.remove(&entity);
    }

    pub fn count_{{id}}(&self) -> usize {
        self.{{id}}.len()
    }

    pub fn clear_{{id}}(&mut self) {
        self.{{id}}.clear();
    }
{{/each}}

    pub fn remove_component(&mut self, entity: EntityId, component_type: ComponentType) {
        match component_type {
{{#each component}}
            component_type::{{id_uppercase}} => self.remove_{{id}}(entity),
{{/each}}
            _ => panic!("Invalid component type: {}", component_type),
        }
    }

    pub fn remove_components(&mut self, entity: EntityId, component_type_set: ComponentTypeSet) {
        for component_type in component_type_set.iter() {
            self.remove_component(entity, component_type);
        }
    }

    pub fn push_component_entity_ids(&self, component_type: ComponentType, ids: &mut Vec<EntityId>) {
        match component_type {
{{#each component}}
    {{#if type}}
            component_type::{{id_uppercase}} => {
                for id in self.{{id}}.keys() {
                    ids.push(*id);
                }
            },
    {{else}}
            component_type::{{id_uppercase}} => {
                for id in self.{{id}}.iter() {
                    ids.push(*id);
                }
            },
    {{/if}}
{{/each}}
            _ => panic!("Invalid component type: {}", component_type),
        }
    }
}

pub struct EcsCtx {
    table: EcsTable,
    tracker: EntityMap<ComponentTypeSet>,
    query_ctx: UnsafeCell<QueryCtx>,
}

impl EcsCtx {
    pub fn new() -> Self {
        EcsCtx {
            table: EcsTable::new(),
            tracker: EntityMap::new(),
            query_ctx: UnsafeCell::new(QueryCtx::new()),
        }
    }

    fn query_ctx_mut(&self) -> &mut QueryCtx {
        unsafe {
            &mut *self.query_ctx.get()
        }
    }

{{#each component}}
    {{#if type}}
    pub fn insert_{{id}}(&mut self, entity: EntityId, value: {{type}}) {
        self.table.insert_{{id}}(entity, value);
        self.tracker.entry(entity).or_insert_with(ComponentTypeSet::new).insert_{{id}}();
        {{#if queried}}
        self.set_dirty_insert_{{id}}();
        {{/if}}
    }

    pub fn contains_{{id}}(&self, entity: EntityId) -> bool {
        self.table.contains_{{id}}(entity)
    }

        {{#if copy}}
    pub fn {{id}}(&self, entity: EntityId) -> Option<{{type}}> {
        self.table.{{id}}(entity)
    }
    pub fn {{id}}_ref(&self, entity: EntityId) -> Option<&{{type}}> {
        self.table.{{id}}_ref(entity)
    }
        {{else}}
            {{#if container}}
    pub fn {{id}}(&self, entity: EntityId) -> Option<&{{container}}<{{type}}>> {
        self.table.{{id}}(entity)
    }
                {{#if RefCell}}
    pub fn {{id}}_borrow(&self, entity: EntityId) -> Option<Ref<{{type}}>> {
        self.table.{{id}}_borrow(entity)
    }
    pub fn {{id}}_borrow_mut(&self, entity: EntityId) -> Option<RefMut<{{type}}>> {
        self.table.{{id}}_borrow_mut(entity)
    }
                {{/if}}
                {{#if UnsafeCell}}
    pub fn {{id}}_unsafe_get_mut(&self, entity: EntityId) -> Option<&mut {{type}}> {
        self.table.{{id}}_unsafe_get_mut(entity)
    }
    pub fn {{id}}_unsafe_get(&self, entity: EntityId) -> Option<&{{type}}> {
        self.table.{{id}}_unsafe_get(entity)
    }
                {{/if}}
            {{else}}
    pub fn {{id}}(&self, entity: EntityId) -> Option<&{{type}}> {
        self.table.{{id}}(entity)
    }
            {{/if}}
        {{/if}}

        {{#if container}}
    pub fn {{id}}_mut(&mut self, entity: EntityId) -> Option<&mut {{container}}<{{type}}>> {
        self.table.{{id}}_mut(entity)
    }
        {{else}}
    pub fn {{id}}_mut(&mut self, entity: EntityId) -> Option<&mut {{type}}> {
        self.table.{{id}}_mut(entity)
    }
        {{/if}}
    {{else}}
    pub fn insert_{{id}}(&mut self, entity: EntityId) {
        self.table.insert_{{id}}(entity);
        self.tracker.entry(entity).or_insert_with(ComponentTypeSet::new).insert_{{id}}();
        {{#if queried}}
        self.set_dirty_insert_{{id}}();
        {{/if}}
    }

    pub fn contains_{{id}}(&self, entity: EntityId) -> bool {
        self.table.contains_{{id}}(entity)
    }
    {{/if}}

    pub fn remove_{{id}}(&mut self, entity: EntityId) {
        self.table.remove_{{id}}(entity);
        let empty = self.tracker.get_mut(&entity).map(|set| {
            set.remove_{{id}}();
            set.is_empty()
        });
        if let Some(true) = empty {
            self.tracker.remove(&entity);
        }
        {{#if queried}}
        self.set_dirty_remove_{{id}}();
        {{/if}}
    }

    {{#if queried}}
    fn set_dirty_insert_{{id}}(&self) {
        self.query_ctx_mut().dirty.{{id}}.insert = true;
    }
    fn set_dirty_remove_{{id}}(&self) {
        self.query_ctx_mut().dirty.{{id}}.remove = true;
    }
    {{/if}}
{{/each}}

    pub fn remove_component(&mut self, entity: EntityId, component_type: ComponentType) {
        match component_type {
{{#each component}}
            component_type::{{id_uppercase}} => self.remove_{{id}}(entity),
{{/each}}
            _ => panic!("Invalid component type: {}", component_type),
        }
    }

    pub fn remove_components(&mut self, entity: EntityId, component_type_set: ComponentTypeSet) {
        for component_type in component_type_set.iter() {
            self.remove_component(entity, component_type);
        }
    }

    pub fn remove_entity(&mut self, entity: EntityId) {
        if let Some(set) = self.tracker.remove(&entity) {
            self.table.remove_components(entity, set);
        }
    }

    pub fn entity(&self, id: EntityId) -> EntityRef {
        EntityRef::new(id, self)
    }

    pub fn entity_mut(&mut self, id: EntityId) -> EntityRefMut {
        EntityRefMut::new(id, self)
    }

{{#each query}}
    pub fn {{id}}(&self) -> {{prefix}}Iter {
        let query_ctx = self.query_ctx_mut();
        if query_ctx.dirty.should_populate_{{id}}() {

            // identify the component with the least number of entities
            let mut _max = usize::MAX;
            let mut component_type = component_type::INVALID_COMPONENT;

    {{#each components}}
            let count = self.table.count_{{id}}();
            if count < _max {
                _max = count;
                component_type = component_type::{{id_uppercase}};
            }
    {{/each}}

            query_ctx.{{id}}.results.clear();

            match component_type {
    {{#each components}}
                component_type::{{id_uppercase}} => {
        {{#if type}}
                    for (id, value) in self.table.{{id}}.iter() {
                        let {{id}} = value as *const {{type}};
            {{#each other_components}}
                        let {{id}} = if let Some(component) =
                {{#if copy}}
                            self.table.{{id}}_ref(*id)
                {{else}}
                            self.table.{{id}}(*id)
                {{/if}}
                        {
                            component as *const {{type}}
                        } else {
                            continue;
                        };
            {{/each}}
                        let result = {{../prefix}}InnerResult {
                            id: *id,
                            {{id}}: {{id}},
            {{#each other_components}}
                            {{id}}: {{id}},
            {{/each}}
                        };
                        query_ctx.{{../id}}.results.push(result);
                    }
        {{else}}
                    for id in self.table.{{id}}.iter() {
            {{#each other_components}}
                        let {{id}} = if let Some(component) =
                {{#if copy}}
                            self.table.{{id}}_ref(*id)
                {{else}}
                            self.table.{{id}}(*id)
                {{/if}}
                        {
                            component as *const {{type}}
                        } else {
                            continue;
                        };
            {{/each}}
                        let result = {{../prefix}}InnerResult {
                            id: *id,
            {{#each other_components}}
                            {{id}}: {{id}},
            {{/each}}
                        };
                        query_ctx.{{../id}}.results.push(result);
                    }
        {{/if}}
                }
    {{/each}}
                _ => panic!("Invalid component type: {}", component_type),
            }

    {{#each components}}
            query_ctx.dirty.{{id}}.clean();
    {{/each}}
        }

        query_ctx.{{id}}.iter()
    }
{{/each}}

    pub fn commit(&mut self, action: &mut EcsAction) {
        self.commit_insertions(&mut action.insertions,
                               &mut action.insertion_types);
        self.commit_removals(&mut action.removals,
                             &mut action.removal_types);
        self.commit_removed_entities(&mut action.removed_entities);

        action.properties.clear();
    }

    fn commit_insertions(&mut self,
                         insertions: &mut ActionInsertionTable,
                         insertion_types: &mut ComponentTypeSet) {
        for component_type in insertion_types.iter() {
            match component_type {
{{#each component}}
                component_type::{{id_uppercase}} => {
    {{#if type}}
                    for (entity_id, value) in insertions.{{id}}.drain() {
                        self.insert_{{id}}(entity_id, value);
                    }
    {{else}}
                    for entity_id in insertions.{{id}}.drain() {
                        self.insert_{{id}}(entity_id);
                    }
    {{/if}}
                }
{{/each}}
                _ => panic!("Invalid component type: {}", component_type),
            }
        }
        insertion_types.clear();
    }

    fn commit_removals(&mut self,
                       removals: &mut ActionRemovalTable,
                       removal_types: &mut ComponentTypeSet) {
        for component_type in removal_types.iter() {
            match component_type {
{{#each component}}
                component_type::{{id_uppercase}} => {
                    for entity_id in removals.{{id}}.drain() {
                        self.remove_{{id}}(entity_id);
                    }
                }
{{/each}}
                _ => panic!("Invalid component type: {}", component_type),
            }
        }
        removal_types.clear();
    }

    fn commit_removed_entities(&mut self, removed_entities: &mut HashSet<EntityId>) {
        for entity_id in removed_entities.drain() {
            self.remove_entity(entity_id);
        }
    }
}

#[derive(Clone, Copy)]
pub struct EntityRef<'a> {
    id: EntityId,
    ctx: &'a EcsCtx,
}

impl<'a> EntityRef<'a> {
    fn new(id: EntityId, ctx: &'a EcsCtx) -> Self {
        EntityRef {
            id: id,
            ctx: ctx,
        }
    }

    pub fn id(self) -> EntityId {
        self.id
    }

    pub fn is_empty(self) -> bool {
        if let Some(set) = self.ctx.tracker.get(&self.id) {
            set.is_empty()
        } else {
            true
        }
    }

{{#each component}}
    pub fn contains_{{id}}(self) -> bool {
        self.ctx.contains_{{id}}(self.id)
    }
    {{#if type}}
        {{#if copy}}
    pub fn {{id}}(self) -> Option<{{type}}> {
        self.ctx.{{id}}(self.id)
    }
    pub fn {{id}}_ref(self) -> Option<&'a {{type}}> {
        self.ctx.{{id}}_ref(self.id)
    }
        {{else}}
            {{#if container}}
    pub fn {{id}}(self) -> Option<&'a {{container}}<{{type}}>> {
        self.ctx.{{id}}(self.id)
    }
                {{#if RefCell}}
    pub fn {{id}}_borrow(self) -> Option<Ref<'a, {{type}}>> {
        self.ctx.{{id}}_borrow(self.id)
    }
    pub fn {{id}}_borrow_mut(self) -> Option<RefMut<'a, {{type}}>> {
        self.ctx.{{id}}_borrow_mut(self.id)
    }
                {{/if}}
                {{#if UnsafeCell}}
    pub fn {{id}}_unsafe_get_mut(self) -> Option<&'a mut {{type}}> {
        self.ctx.{{id}}_unsafe_get_mut(self.id)
    }
    pub fn {{id}}_unsafe_get(self) -> Option<&'a {{type}}> {
        self.ctx.{{id}}_unsafe_get(self.id)
    }
                {{/if}}
            {{else}}
    pub fn {{id}}(self) -> Option<&'a {{type}}> {
        self.ctx.{{id}}(self.id)
    }
            {{/if}}
        {{/if}}
    {{/if}}
{{/each}}
}

pub trait EntityPopulate {
{{#each component}}
    {{#if type}}
    fn insert_{{id}}(&mut self, value: {{type}});
    {{else}}
    fn insert_{{id}}(&mut self);
    {{/if}}
{{/each}}
}

pub struct EntityRefMut<'a> {
    id: EntityId,
    ctx: &'a mut EcsCtx,
}

impl<'a> EntityRefMut<'a> {
    fn new(id: EntityId, ctx: &'a mut EcsCtx) -> Self {
        EntityRefMut {
            id: id,
            ctx: ctx,
        }
    }

    pub fn id(&self) -> EntityId {
        self.id
    }

    pub fn is_empty(&self) -> bool {
        if let Some(set) = self.ctx.tracker.get(&self.id) {
            set.is_empty()
        } else {
            true
        }
    }

    pub fn destroy(self) {
        self.ctx.remove_entity(self.id);
    }
{{#each component}}
    pub fn contains_{{id}}(&self) -> bool {
        self.ctx.contains_{{id}}(self.id)
    }
    pub fn remove_{{id}}(&mut self) {
        self.ctx.remove_{{id}}(self.id)
    }
    {{#if type}}
        {{#if copy}}
    pub fn {{id}}(&self) -> Option<{{type}}> {
        self.ctx.{{id}}(self.id)
    }
    pub fn {{id}}_ref(&self) -> Option<&{{type}}> {
        self.ctx.{{id}}_ref(self.id)
    }
        {{else}}
            {{#if container}}
    pub fn {{id}}(&self) -> Option<&{{container}}<{{type}}>> {
        self.ctx.{{id}}(self.id)
    }
                {{#if RefCell}}
    pub fn {{id}}_borrow(&self) -> Option<Ref<{{type}}>> {
        self.ctx.{{id}}_borrow(self.id)
    }
    pub fn {{id}}_borrow_mut(&self) -> Option<RefMut<{{type}}>> {
        self.ctx.{{id}}_borrow_mut(self.id)
    }
                {{/if}}
                {{#if UnsafeCell}}
    pub fn {{id}}_unsafe_get_mut(&self) -> Option<&mut {{type}}> {
        self.ctx.{{id}}_unsafe_get_mut(self.id)
    }
    pub fn {{id}}_unsafe_get(&self) -> Option<&{{type}}> {
        self.ctx.{{id}}_unsafe_get(self.id)
    }
                {{/if}}
            {{else}}
    pub fn {{id}}(&self) -> Option<&{{type}}> {
        self.ctx.{{id}}(self.id)
    }
            {{/if}}
        {{/if}}
        {{#if container}}
    pub fn {{id}}_mut(&mut self) -> Option<&mut {{container}}<{{type}}>> {
        self.ctx.{{id}}_mut(self.id)
    }
        {{else}}
    pub fn {{id}}_mut(&mut self) -> Option<&mut {{type}}> {
        self.ctx.{{id}}_mut(self.id)
    }
        {{/if}}
    {{/if}}
{{/each}}
}

impl<'a> EntityPopulate for EntityRefMut<'a> {
{{#each component}}
    {{#if type}}
    fn insert_{{id}}(&mut self, value: {{type}}) {
        self.ctx.insert_{{id}}(self.id, value);
    }
    {{else}}
    fn insert_{{id}}(&mut self) {
        self.ctx.insert_{{id}}(self.id);
    }
    {{/if}}
{{/each}}
}

impl<'a> EntityPopulate for ActionEntityRefMut<'a> {
{{#each component}}
    {{#if type}}
    fn insert_{{id}}(&mut self, value: {{type}}) {
        self.action.insert_{{id}}(self.id, value);
    }
    {{else}}
    fn insert_{{id}}(&mut self) {
        self.action.insert_{{id}}(self.id);
    }
    {{/if}}
{{/each}}
}

{{#each query}}
pub struct {{prefix}}Result<'a> {
    id: EntityId,
    {{#each components}}
        {{#if type}}
    {{id}}: &'a {{type}},
        {{/if}}
    {{/each}}
}

pub struct {{prefix}}Iter<'a> {
    slice_iter: slice::Iter<'a, {{prefix}}InnerResult>,
}

impl<'a> Iterator for {{prefix}}Iter<'a> {
    type Item = {{prefix}}Result<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.slice_iter.next().map(|inner| {
            inner.to_outer_result()
        })
    }
}

struct {{prefix}}InnerResult {
    id: EntityId,
    {{#each components}}
        {{#if type}}
    {{id}}: *const {{type}},
        {{/if}}
    {{/each}}
}

impl {{prefix}}InnerResult {
    fn to_outer_result(&self) -> {{prefix}}Result {
        unsafe {
            {{prefix}}Result {
                id: self.id,
{{#each components}}
    {{#if type}}
                {{id}}: &(*self.{{id}}),
    {{/if}}
{{/each}}
            }
        }
    }
}

struct {{prefix}}QueryCtx {
    results: Vec<{{prefix}}InnerResult>,
}

impl {{prefix}}QueryCtx {
    fn new() -> Self {
        {{prefix}}QueryCtx {
            results: Vec::new(),
        }
    }

    fn iter(&self) -> {{prefix}}Iter {
        {{prefix}}Iter {
            slice_iter: self.results.iter(),
        }
    }
}
{{/each}}

struct QueryCtx {
    dirty: DirtyComponentTracker,
{{#each query}}
    {{id}}: {{prefix}}QueryCtx,
{{/each}}
}

impl QueryCtx {
    fn new() -> Self {
        QueryCtx {
            dirty: DirtyComponentTracker::new(),
{{#each query}}
            {{id}}: {{prefix}}QueryCtx::new(),
{{/each}}
        }
    }
}

pub struct ActionInsertionTable {
{{#each component}}
    {{#if type}}
    pub {{id}}: HashMap<EntityId, {{type}}>,
    {{else}}
    pub {{id}}: HashSet<EntityId>,
    {{/if}}
{{/each}}
}

impl ActionInsertionTable {
    fn new() -> Self {
        ActionInsertionTable {
{{#each component}}
    {{#if type}}
            {{id}}: HashMap::new(),
    {{else}}
            {{id}}: HashSet::new(),
    {{/if}}
{{/each}}
        }
    }
}

pub struct ActionRemovalTable {
{{#each component}}
    pub {{id}}: HashSet<EntityId>,
{{/each}}
}

impl ActionRemovalTable {
    fn new() -> Self {
        ActionRemovalTable {
{{#each component}}
            {{id}}: HashSet::new(),
{{/each}}
        }
    }
}

pub struct EcsAction {
    pub insertions: ActionInsertionTable,
    pub insertion_types: ComponentTypeSet,
    pub removals: ActionRemovalTable,
    pub removal_types: ComponentTypeSet,
    pub removed_entities: HashSet<EntityId>,
    pub properties: EcsActionProperties,
}

impl EcsAction {
    pub fn new() -> Self {
        EcsAction {
            insertions: ActionInsertionTable::new(),
            insertion_types: ComponentTypeSet::new(),
            removals: ActionRemovalTable::new(),
            removal_types: ComponentTypeSet::new(),
            removed_entities: HashSet::new(),
            properties: EcsActionProperties::new(),
        }
    }

{{#each component}}
    {{#if type}}
    pub fn insert_{{id}}(&mut self, entity: EntityId, value: {{type}}) {
        self.insertions.{{id}}.insert(entity, value);
        self.insertion_types.insert_{{id}}();
    }
    {{else}}
    pub fn insert_{{id}}(&mut self, entity: EntityId) {
        self.insertions.{{id}}.insert(entity);
        self.insertion_types.insert_{{id}}();
    }
    {{/if}}
    pub fn remove_{{id}}(&mut self, entity: EntityId) {
        self.removals.{{id}}.insert(entity);
        self.removal_types.insert_{{id}}();
    }
{{/each}}
    pub fn remove_entity(&mut self, entity: EntityId) {
        self.removed_entities.insert(entity);
    }
{{#each action_property}}
    {{#if type}}
    pub fn set_{{id}}(&mut self, value: {{type}}) {
        self.properties.insert_{{id}}(value);
    }
    {{else}}
    pub fn set_{{id}}(&mut self) {
        self.properties.insert_{{id}}();
    }
    {{/if}}
    pub fn clear_{{id}}(&mut self) {
        self.properties.remove_{{id}}();
    }
{{/each}}

    pub fn entity_mut(&mut self, id: EntityId) -> ActionEntityRefMut {
        ActionEntityRefMut::new(id, self)
    }
}

pub struct ActionEntityRefMut<'a> {
    id: EntityId,
    action: &'a mut EcsAction,
}

impl<'a> ActionEntityRefMut<'a> {
    fn new(id: EntityId, action: &'a mut EcsAction) -> Self {
        ActionEntityRefMut {
            id: id,
            action: action,
        }
    }
}

pub const NUM_ACTION_PROPERTIES: usize = {{num_action_properties}};
const ACTION_PROPERTY_TYPE_SET_NUM_WORDS: usize = {{component_set_num_words}};

pub type ActionPropertyType = usize;

pub mod action_property_type {
    use std::usize;

{{#each action_property}}
    pub const {{id_uppercase}}: usize = {{index}};
{{/each}}
}

pub struct ActionPropertyTypeSet {
    bitfields: [usize; ACTION_PROPERTY_TYPE_SET_NUM_WORDS],
}

pub struct ActionPropertyTypeSetIter {
    bitfields: [usize; ACTION_PROPERTY_TYPE_SET_NUM_WORDS],
    index: usize,
}

impl ActionPropertyTypeSetIter {
    fn new(bitfields: [usize; ACTION_PROPERTY_TYPE_SET_NUM_WORDS]) -> Self {
        ActionPropertyTypeSetIter {
            bitfields: bitfields,
            index: 0,
        }
    }
}

impl Iterator for ActionPropertyTypeSetIter {
    type Item = ActionPropertyType;
    fn next(&mut self) -> Option<Self::Item> {
        while self.index < ACTION_PROPERTY_TYPE_SET_NUM_WORDS && self.bitfields[self.index] == 0 {
            self.index += 1;
        }
        if self.index == ACTION_PROPERTY_TYPE_SET_NUM_WORDS {
            return None;
        }

        let trailing = self.bitfields[self.index].trailing_zeros() as usize;
        self.bitfields[self.index] &= !(1 << trailing);
        Some(self.index * WORD_BITS + trailing)
    }
}

impl ActionPropertyTypeSet {
    pub fn new() -> Self {
        ActionPropertyTypeSet {
            bitfields: [0; COMPONENT_TYPE_SET_NUM_WORDS],
        }
    }

    pub fn is_empty(&self) -> bool {
        for b in &self.bitfields {
            if *b != 0 {
                return false;
            }
        }

        true
    }

    pub fn clear(&mut self) {
        for b in &mut self.bitfields {
            *b = 0;
        }
    }

    pub fn iter(&self) -> ActionPropertyTypeSetIter {
        ActionPropertyTypeSetIter::new(self.bitfields)
    }

{{#each action_property}}
    pub fn contains_{{id}}(&self) -> bool {
        self.bitfields[{{set_index}}] & (1 << {{set_bit}}) != 0
    }

    pub fn insert_{{id}}(&mut self) {
        self.bitfields[{{set_index}}] |= 1 << {{set_bit}};
    }

    pub fn remove_{{id}}(&mut self) {
        self.bitfields[{{set_index}}] &= !(1 << {{set_bit}});
    }
{{/each}}
}

pub struct EcsActionProperties {
    property_types: ActionPropertyTypeSet,
{{#each action_property}}
    {{#if type}}
    pub {{id}}: Option<{{type}}>,
    {{else}}
    pub {{id}}: bool,
    {{/if}}
{{/each}}
}

impl EcsActionProperties {
    pub fn new() -> Self {
        EcsActionProperties {
            property_types: ActionPropertyTypeSet::new(),
{{#each action_property}}
    {{#if type}}
            {{id}}: None,
    {{else}}
            {{id}}: false,
    {{/if}}
{{/each}}
        }
    }

    pub fn clear(&mut self) {
        for property_type in self.property_types.iter() {
            match property_type {
{{#each action_property}}
                action_property_type::{{id_uppercase}} => {
    {{#if type}}
                    self.{{id}} = None;
    {{else}}
                    self.{{id}} = false;
    {{/if}}
                }
{{/each}}
                _ => panic!("Invalid action property type: {}", property_type),
            }
        }
        self.property_types.clear();
    }

{{#each action_property}}
    {{#if type}}
    pub fn insert_{{id}}(&mut self, value: {{type}}) {
        self.{{id}} = Some(value);
    }
        {{#if copy}}
    pub fn {{id}}(&self) -> Option<{{type}}> {
        self.{{id}}
    }
    pub fn {{id}}_ref(&self) -> Option<&{{type}}> {
        self.{{id}}.as_ref()
    }
        {{else}}
    pub fn {{id}}(&self) -> Option<&{{type}}> {
        self.{{id}}.as_ref()
    }
        {{/if}}
    pub fn contains_{{id}}(&self) -> bool {
        self.{{id}}.is_some()
    }
    {{else}}
    pub fn insert_{{id}}(&mut self) {
        self.{{id}} = true;
    }
    pub fn contains_{{id}}(&self) -> bool {
        self.{{id}}
    }
    {{/if}}
    pub fn remove_{{id}}(&mut self) {
    {{#if type}}
        self.{{id}} = None;
    {{else}}
        self.{{id}} = false;
    {{/if}}
    }
{{/each}}
}
"#;

fn generate_code(mut toml: String) -> String {
    // turn the toml string into json for compatibility with handlebars
    let mut json = Toml::as_json(&mut toml).unwrap();

    let mut component_clones = HashMap::new();

    let num_components = json.search("component").unwrap().as_object().unwrap().len();
    json.as_object_mut().unwrap().insert("num_components".to_string(), Json::U64(num_components as u64));

    let word_size = mem::size_of::<usize>();
    json.as_object_mut().unwrap().insert("word_size".to_string(), Json::U64(word_size as u64));

    let word_bits = word_size * 8;
    json.as_object_mut().unwrap().insert("word_bits".to_string(), Json::U64(word_bits as u64));

    let component_set_num_words = (num_components - 1) / word_bits + 1;
    json.as_object_mut().unwrap().insert("component_set_num_words".to_string(), Json::U64(component_set_num_words as u64));

    let mut queried_components = json::Object::new();
    if let Some(query) = json.as_object().unwrap().get("query") {
        for query in query.as_object().unwrap().values() {
            let query_obj = query.as_object().unwrap();
            let components_json = query_obj.get("components").unwrap();
            let components_arr = components_json.as_array().unwrap();
            for component in components_arr {
                let component_str = component.as_string().unwrap();
                queried_components.insert(component_str.to_string(), Json::Boolean(true));
            }
        }
    }

    let mut index = 0;
    for (id, component) in json.as_object_mut().unwrap().get_mut("component").unwrap().as_object_mut().unwrap().iter_mut() {
        let component_obj = component.as_object_mut().unwrap();
        component_obj.insert("index".to_string(), Json::U64(index as u64));
        component_obj.insert("set_index".to_string(), Json::U64((index / word_bits) as u64));
        component_obj.insert("set_bit".to_string(), Json::U64((index % word_bits) as u64));
        component_obj.insert("id".to_string(), Json::String(id.to_string()));
        component_obj.insert("id_uppercase".to_string(), Json::String(id.to_uppercase()));

        let maybe_container = component_obj.get("container").map(|container| {
            container.clone()
        });

        if let Some(container) = maybe_container {
            component_obj.insert(container.as_string().unwrap().to_string(), Json::Boolean(true));
        }

        if queried_components.contains_key(id) {
            component_obj.insert("queried".to_string(), Json::Boolean(true));
        }

        component_clones.insert(id.to_string(), component_obj.clone());

        index += 1;
    }

    if let Some(mut query) = json.as_object_mut().unwrap().get_mut("query") {
        for (id, query) in query.as_object_mut().unwrap().iter_mut() {
            let query_obj = query.as_object_mut().unwrap();
            query_obj.insert("id".to_string(), Json::String(id.to_string()));
            let components_json = query_obj.remove("components").unwrap();
            let component_names = components_json.as_array().unwrap();
            let mut component_clone_arr = json::Array::new();
            for component in component_names {
                let component_str = component.as_string().unwrap();
                let mut clone = component_clones.get(component_str).unwrap().clone();
                let no_type = clone.get("type").is_none();
                let mut result_components = json::Array::new();
                for component_inner in component_names {
                    let component_inner_str = component_inner.as_string().unwrap();
                    if no_type || component_inner_str != component_str {
                        let other_clone = component_clones.get(component_inner_str).unwrap().clone();
                        if other_clone.get("type").is_some() {
                            result_components.push(Json::Object(other_clone));
                        }
                    }
                }
                clone.insert("other_components".to_string(), Json::Array(result_components));
                component_clone_arr.push(Json::Object(clone));
            }
            query_obj.insert("components".to_string(), Json::Array(component_clone_arr));
        }
    }

    json.as_object_mut().unwrap().insert("queried_components".to_string(), Json::Object(queried_components));

    let num_action_properties = if let Some(action_property) = json.search("action_property") {
        action_property.as_object().unwrap().len()
    } else {
        0
    };

    json.as_object_mut().unwrap().insert("num_action_properties".to_string(), Json::U64(num_action_properties as u64));
    index = 0;

    if let Some(mut action_property) = json.as_object_mut().unwrap().get_mut("action_property") {
        for (id, action_property) in action_property.as_object_mut().unwrap().iter_mut() {
            let action_property_obj = action_property.as_object_mut().unwrap();
            action_property_obj.insert("index".to_string(), Json::U64(index as u64));
            action_property_obj.insert("set_index".to_string(), Json::U64((index / word_bits) as u64));
            action_property_obj.insert("set_bit".to_string(), Json::U64((index % word_bits) as u64));
            action_property_obj.insert("id".to_string(), Json::String(id.to_string()));
            action_property_obj.insert("id_uppercase".to_string(), Json::String(id.to_uppercase()));

            index += 1;
        }
    }

    let mut handlebars = Handlebars::new();

    // prevent xml escaping
    handlebars.register_escape_fn(|input| input.to_string());
    handlebars.template_render(TEMPLATE, &json).unwrap()
}

fn read_file_to_string<P: AsRef<Path>>(path: P) -> String {
    let mut file = File::open(path).unwrap();
    let mut string = String::new();
    file.read_to_string(&mut string).unwrap();

    string
}

pub fn generate_ecs<P: AsRef<Path>, Q: AsRef<Path>>(in_path: P, out_path: Q) {

    let string = read_file_to_string(in_path);

    let output_string = generate_code(string);

    let mut outfile = File::create(out_path).unwrap();
    write!(outfile, "{}", output_string).unwrap();
}
