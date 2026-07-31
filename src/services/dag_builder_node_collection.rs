// DagBuilder node collection: collect_nodes from FileContext AST items

impl DagBuilder {
    fn collect_nodes(&mut self, file: &FileContext) {
        // Register the file's own module node in the FIRST pass so that the second
        // pass can resolve `use` paths to it (#653: import edges pointed at invented
        // ids and were then silently dropped by finalize_graph, giving "0 edges").
        self.collect_file_module_node(file);

        // Build complexity lookup maps for O(1) access
        let function_complexity: FxHashMap<&str, u32> = file
            .complexity_metrics
            .as_ref()
            .map(|m| {
                m.functions
                    .iter()
                    .map(|f| (f.name.as_str(), u32::from(f.metrics.cognitive)))
                    .collect()
            })
            .unwrap_or_default();

        let class_complexity: FxHashMap<&str, u32> = file
            .complexity_metrics
            .as_ref()
            .map(|m| {
                m.classes
                    .iter()
                    .map(|c| (c.name.as_str(), u32::from(c.metrics.cognitive)))
                    .collect()
            })
            .unwrap_or_default();

        for item in &file.items {
            self.collect_single_node(item, file, &function_complexity, &class_complexity);
        }
    }

    fn collect_single_node(
        &mut self,
        item: &AstItem,
        file: &FileContext,
        function_complexity: &FxHashMap<&str, u32>,
        class_complexity: &FxHashMap<&str, u32>,
    ) {
        match item {
            AstItem::Function {
                name,
                line,
                visibility: _,
                is_async: _,
            } => {
                self.collect_function_node(name, *line, file, function_complexity);
            }
            AstItem::Struct {
                name,
                line,
                fields_count,
                derives: _,
                visibility: _,
            } => {
                self.collect_struct_node(name, *line, *fields_count, file, class_complexity);
            }
            AstItem::Trait {
                name,
                line,
                visibility: _,
            } => {
                self.collect_trait_node(name, *line, file);
            }
            AstItem::Module {
                name,
                line,
                visibility: _,
            } => {
                self.collect_module_node(name, *line, file);
            }
            _ => {}
        }
    }

    /// Create the module node that represents the file itself and index it under
    /// every suffix of its module path (`helpers`, `utils::helpers`, ...).
    fn collect_file_module_node(&mut self, file: &FileContext) {
        let id = self.normalize_path(&file.path);
        let node = NodeInfo {
            id: id.clone(),
            label: self.extract_module_name(&file.path),
            node_type: NodeType::Module,
            file_path: file.path.clone(),
            line_number: 0,
            complexity: file
                .complexity_metrics
                .as_ref()
                .map_or(1, |m| u32::from(m.total_complexity.cognitive)),
            metadata: FxHashMap::default(),
        };
        self.add_node(self.enrich_node(node));

        let segments = module_path_segments(&file.path);
        for start in 0..segments.len() {
            let key = segments[start..].join("::");
            let entry = self.module_map.entry(key).or_default();
            if !entry.contains(&id) {
                entry.push(id.clone());
            }
        }
    }

    fn collect_function_node(
        &mut self,
        name: &str,
        line: usize,
        file: &FileContext,
        function_complexity: &FxHashMap<&str, u32>,
    ) {
        let id = format!("{}::{}", self.normalize_path(&file.path), name);
        let node = NodeInfo {
            id: id.clone(),
            label: name.to_string(),
            node_type: NodeType::Function,
            file_path: file.path.clone(),
            line_number: line,
            complexity: function_complexity
                .get(name)
                .copied()
                .unwrap_or(2), // Default function complexity
            metadata: FxHashMap::default(),
        };
        self.add_node(self.enrich_node(node));
        self.function_map.insert(name.to_string(), id);
    }

    fn collect_struct_node(
        &mut self,
        name: &str,
        line: usize,
        fields_count: usize,
        file: &FileContext,
        class_complexity: &FxHashMap<&str, u32>,
    ) {
        let id = format!("{}::{}", self.normalize_path(&file.path), name);
        let node = NodeInfo {
            id: id.clone(),
            label: name.to_string(),
            node_type: NodeType::Class,
            file_path: file.path.clone(),
            line_number: line,
            complexity: class_complexity
                .get(name)
                .copied()
                .unwrap_or(fields_count as u32 + 1),
            metadata: FxHashMap::default(),
        };
        self.add_node(self.enrich_node(node));
        self.type_map.insert(name.to_string(), id);
    }

    fn collect_trait_node(
        &mut self,
        name: &str,
        line: usize,
        file: &FileContext,
    ) {
        let id = format!("{}::{}", self.normalize_path(&file.path), name);
        let node = NodeInfo {
            id: id.clone(),
            label: name.to_string(),
            node_type: NodeType::Trait,
            file_path: file.path.clone(),
            line_number: line,
            complexity: 1,
            metadata: FxHashMap::default(),
        };
        self.add_node(self.enrich_node(node));
        self.type_map.insert(name.to_string(), id);
    }

    fn collect_module_node(
        &mut self,
        name: &str,
        line: usize,
        file: &FileContext,
    ) {
        let id = format!("{}::{}", self.normalize_path(&file.path), name);
        let node = NodeInfo {
            id: id.clone(),
            label: name.to_string(),
            node_type: NodeType::Module,
            file_path: file.path.clone(),
            line_number: line,
            complexity: 1,
            metadata: FxHashMap::default(),
        };
        self.add_node(self.enrich_node(node));
    }
}
