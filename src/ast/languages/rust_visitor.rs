/// Visitor for converting syn AST to unified AST
struct RustAstVisitor<'a> {
    dag: &'a mut AstDag,
    current_parent: Option<u32>,
}

impl<'a> RustAstVisitor<'a> {
    fn new(dag: &'a mut AstDag) -> Self {
        Self {
            dag,
            current_parent: None,
        }
    }

    
    fn add_node(&mut self, kind: AstKind) -> u32 {
        let mut node = UnifiedAstNode::new(kind, Language::Rust);

        // Set parent if we have one
        if let Some(parent) = self.current_parent {
            node.parent = parent;
        }

        self.dag.add_node(node)
    }
}

impl Visit<'_> for RustAstVisitor<'_> {
    fn visit_item_fn(&mut self, node: &ItemFn) {
        let mut ast_node =
            UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);

        // Set async flag if needed
        if node.sig.asyncness.is_some() {
            ast_node.flags.set(NodeFlags::ASYNC);
        }

        let key = self.dag.add_node(ast_node);

        let old_parent = self.current_parent;
        self.current_parent = Some(key);

        syn::visit::visit_item_fn(self, node);

        self.current_parent = old_parent;
    }

    fn visit_item_struct(&mut self, node: &ItemStruct) {
        let ast_node = UnifiedAstNode::new(AstKind::Class(ClassKind::Struct), Language::Rust);

        let key = self.dag.add_node(ast_node);

        let old_parent = self.current_parent;
        self.current_parent = Some(key);

        syn::visit::visit_item_struct(self, node);

        self.current_parent = old_parent;
    }

    fn visit_item_enum(&mut self, node: &ItemEnum) {
        let ast_node = UnifiedAstNode::new(AstKind::Class(ClassKind::Enum), Language::Rust);

        let key = self.dag.add_node(ast_node);

        let old_parent = self.current_parent;
        self.current_parent = Some(key);

        syn::visit::visit_item_enum(self, node);

        self.current_parent = old_parent;
    }

    fn visit_item_trait(&mut self, node: &ItemTrait) {
        let ast_node = UnifiedAstNode::new(AstKind::Class(ClassKind::Trait), Language::Rust);

        let key = self.dag.add_node(ast_node);

        let old_parent = self.current_parent;
        self.current_parent = Some(key);

        syn::visit::visit_item_trait(self, node);

        self.current_parent = old_parent;
    }

    fn visit_item_impl(&mut self, node: &ItemImpl) {
        // For impl blocks, we create a special kind
        let ast_node = UnifiedAstNode::new(AstKind::Class(ClassKind::Regular), Language::Rust);

        let key = self.dag.add_node(ast_node);

        let old_parent = self.current_parent;
        self.current_parent = Some(key);

        syn::visit::visit_item_impl(self, node);

        self.current_parent = old_parent;
    }
}
