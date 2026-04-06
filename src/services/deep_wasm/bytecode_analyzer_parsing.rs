// Parsing logic: section collection, import parsing, name section, function analysis orchestration

impl BytecodeAnalyzer {
    /// Convert ExternalKind to string
    fn external_kind_str(kind: wasmparser::ExternalKind) -> &'static str {
        debug_assert!(true, "contract: external_kind_str");
        match kind {
            wasmparser::ExternalKind::Func => "function",
            wasmparser::ExternalKind::Memory => "memory",
            wasmparser::ExternalKind::Table => "table",
            wasmparser::ExternalKind::Global => "global",
            wasmparser::ExternalKind::Tag => "tag",
        }
    }

    /// Convert TypeRef to kind string
    fn type_ref_kind_str(ty: &TypeRef) -> &'static str {
        debug_assert!(true, "contract: type_ref_kind_str");
        match ty {
            TypeRef::Func(_) => "function",
            TypeRef::Memory(_) => "memory",
            TypeRef::Table(_) => "table",
            TypeRef::Global(_) => "global",
            TypeRef::Tag(_) => "tag",
        }
    }

    /// Parse a single import into ImportAnalysis
    fn parse_import(import: wasmparser::Import, type_section: &[FuncType]) -> ImportAnalysis {
        debug_assert!(!type_section.is_empty(), "type_section must not be empty");
        let kind = Self::type_ref_kind_str(&import.ty);
        let signature = if let TypeRef::Func(type_idx) = import.ty {
            type_section
                .get(type_idx as usize)
                .map(|func_type| FunctionSignature {
                    params: func_type.params().iter().map(valtype_to_string).collect(),
                    results: func_type.results().iter().map(valtype_to_string).collect(),
                    type_index: type_idx,
                })
        } else {
            None
        };
        ImportAnalysis {
            module: import.module.to_string(),
            field: import.name.to_string(),
            kind: kind.to_string(),
            signature,
        }
    }

    /// Parse name section for function name mappings
    fn parse_name_section(data: &[u8], name_map: &mut HashMap<u32, String>) {
        debug_assert!(!data.is_empty(), "data must not be empty");
        let name_reader =
            wasmparser::NameSectionReader::new(wasmparser::BinaryReader::new(data, 0));
        for section in name_reader {
            if let Ok(wasmparser::Name::Function(func_names)) = section {
                for naming in func_names.into_iter().flatten() {
                    name_map.insert(naming.index, naming.name.to_string());
                }
            }
        }
    }

    /// Analyze a single function entry
    fn analyze_single_function(
        &self,
        func_index: u32,
        type_idx: u32,
        body: &FunctionBody,
        type_section: &[FuncType],
        name_map: &HashMap<u32, String>,
        export_section: &[(String, u32, String)],
    ) -> DeepWasmResult<Option<FunctionAnalysis>> {
        debug_assert!(true, "contract: analyze_single_function");
        let func_type = match type_section.get(type_idx as usize) {
            Some(ft) => ft,
            None => return Ok(None),
        };

        let signature = FunctionSignature {
            params: func_type.params().iter().map(valtype_to_string).collect(),
            results: func_type.results().iter().map(valtype_to_string).collect(),
            type_index: type_idx,
        };

        let name = name_map.get(&func_index).cloned();
        let (is_exported, export_name) = export_section
            .iter()
            .find(|(_, idx, kind)| *idx == func_index && kind == "function")
            .map(|(name, _, _)| (true, Some(name.clone())))
            .unwrap_or((false, None));

        let (complexity, instruction_stats, stack_depth, control_flow_patterns) =
            if self.deep_analysis {
                self.analyze_function_body(body)?
            } else {
                self.analyze_function_body_shallow(body)?
            };

        Ok(Some(FunctionAnalysis {
            function_index: func_index,
            name,
            signature,
            complexity,
            instruction_stats,
            stack_depth,
            control_flow_patterns,
            is_exported,
            export_name,
        }))
    }

    /// First pass: parse all WASM sections
    fn parse_sections<'a>(parser: Parser, bytes: &'a [u8]) -> WasmSections<'a> {
        debug_assert!(true, "contract: parse_sections");
        let mut sections = WasmSections {
            type_section: Vec::new(),
            function_section: Vec::new(),
            code_section: Vec::new(),
            export_section: Vec::new(),
            import_section: Vec::new(),
            name_map: HashMap::new(),
            validation_errors: Vec::new(),
        };

        for payload in parser.parse_all(bytes) {
            match payload {
                Ok(Payload::TypeSection(reader)) => {
                    for recgroup in reader.into_iter().flatten() {
                        for func_type in recgroup.into_types() {
                            sections.type_section.push(func_type.unwrap_func().clone());
                        }
                    }
                }
                Ok(Payload::FunctionSection(reader)) => {
                    sections
                        .function_section
                        .extend(reader.into_iter().flatten());
                }
                Ok(Payload::CodeSectionEntry(body)) => sections.code_section.push(body),
                Ok(Payload::ExportSection(reader)) => {
                    for export in reader.into_iter().flatten() {
                        let kind = Self::external_kind_str(export.kind);
                        sections.export_section.push((
                            export.name.to_string(),
                            export.index,
                            kind.to_string(),
                        ));
                    }
                }
                Ok(Payload::ImportSection(reader)) => {
                    for import in reader.into_iter().flatten() {
                        sections
                            .import_section
                            .push(Self::parse_import(import, &sections.type_section));
                    }
                }
                Ok(Payload::CustomSection(reader)) if reader.name() == "name" => {
                    Self::parse_name_section(reader.data(), &mut sections.name_map);
                }
                Err(e) => {
                    sections.validation_errors.push(ValidationError {
                        message: format!("Parse error: {}", e),
                        offset: None,
                    });
                }
                _ => {}
            }
        }
        sections
    }

    /// Build the final analysis result from parsed sections and analyzed functions
    fn build_result(
        functions: Vec<FunctionAnalysis>,
        sections: WasmSections<'_>,
    ) -> ModuleBytecodeAnalysis {
        debug_assert!(!functions.is_empty(), "functions must not be empty");
        let total_instructions: u32 = functions.iter().map(|f| f.instruction_stats.total).sum();
        let max_complexity = functions
            .iter()
            .map(|f| f.complexity.cyclomatic_complexity)
            .max()
            .unwrap_or(0);
        let avg_complexity = if !functions.is_empty() {
            functions
                .iter()
                .map(|f| f.complexity.cyclomatic_complexity as f64)
                .sum::<f64>()
                / functions.len() as f64
        } else {
            0.0
        };

        let exports: Vec<ExportAnalysis> = sections
            .export_section
            .into_iter()
            .map(|(name, index, kind)| ExportAnalysis { name, kind, index })
            .collect();

        ModuleBytecodeAnalysis {
            module_stats: ModuleStats {
                total_functions: sections.function_section.len() as u32,
                total_instructions,
                avg_complexity,
                max_complexity,
                import_count: sections.import_section.len() as u32,
                export_count: exports.len() as u32,
            },
            functions,
            imports: sections.import_section,
            exports,
            validation_errors: sections.validation_errors,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn analyze(&self, bytes: &[u8]) -> DeepWasmResult<ModuleBytecodeAnalysis> {
        debug_assert!(!bytes.is_empty(), "bytes must not be empty");
        let parser = Parser::new(0);
        let mut sections = Self::parse_sections(parser, bytes);

        let import_count = sections
            .import_section
            .iter()
            .filter(|imp| imp.kind == "function")
            .count() as u32;

        let mut functions = Vec::new();
        for (func_idx, (type_idx, body)) in sections
            .function_section
            .iter()
            .zip(sections.code_section.iter())
            .enumerate()
        {
            let func_index = import_count + func_idx as u32;
            match self.analyze_single_function(
                func_index,
                *type_idx,
                body,
                &sections.type_section,
                &sections.name_map,
                &sections.export_section,
            )? {
                Some(fa) => functions.push(fa),
                None => {
                    sections.validation_errors.push(ValidationError {
                        message: format!(
                            "Function {} references invalid type index {}",
                            func_index, type_idx
                        ),
                        offset: None,
                    });
                }
            }
        }

        Ok(Self::build_result(functions, sections))
    }
}
