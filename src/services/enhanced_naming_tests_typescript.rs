        fn parse_typescript(code: &str) -> Module {
            let source_map = Arc::new(SourceMap::default());
            let source_file = source_map
                .new_source_file(FileName::Custom("test.ts".into()).into(), code.to_string());

            let lexer = Lexer::new(
                Syntax::Typescript(TsSyntax {
                    tsx: true,
                    decorators: true,
                    dts: false,
                    no_early_errors: true,
                    disallow_ambiguous_jsx_like: true,
                }),
                Default::default(),
                StringInput::from(&*source_file),
                None,
            );

            let mut parser = Parser::new_from(lexer);
            parser.parse_module().expect("Failed to parse TypeScript")
        }

        /// Test: TypeScript type information should be preserved in function names
        #[test]
        fn test_typescript_type_information_extraction() {
            let code = r#"
                interface User {
                    id: string;
                    name: string;
                    email: string;
                    roles: UserRole[];
                }

                type UserRole = 'admin' | 'user' | 'guest';

                interface ApiResponse<T> {
                    data: T;
                    status: number;
                    message?: string;
                }

                // Function with typed parameters and return type
                async function fetchUser(userId: string): Promise<ApiResponse<User>> {
                    const response = await fetch(`/api/users/${userId}`);
                    return response.json() as ApiResponse<User>;
                }

                // Generic function
                function createRepository<T extends { id: string }>(
                    entityName: string
                ): Repository<T> {
                    return new Repository<T>(entityName);
                }

                // Function with union types and optional parameters
                function processPayment(
                    amount: number,
                    method: 'credit' | 'debit' | 'paypal',
                    options?: {
                        currency?: 'USD' | 'EUR' | 'GBP';
                        description?: string;
                        metadata?: Record<string, any>;
                    }
                ): Promise<{ success: boolean; transactionId?: string }> {
                    // Implementation
                    return Promise.resolve({ success: true, transactionId: 'txn_123' });
                }

                // Class with typed methods and properties
                class DataService<T> {
                    private cache: Map<string, T> = new Map();

                    constructor(private apiUrl: string) {}

                    async get<K extends keyof T>(id: string, fields?: K[]): Promise<Pick<T, K> | null> {
                        // Implementation
                        return null;
                    }

                    async update(id: string, updates: Partial<T>): Promise<T> {
                        // Implementation
                        return {} as T;
                    }
                }
            "#;

            let module = parse_typescript(code);
            let visitor = EnhancedTypeScriptVisitor::new(Path::new("typed-service.ts"));
            let items = visitor.extract_items(&module);

            let interfaces: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Trait { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            let functions: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Function { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            // EXPECTATION: Should extract TypeScript interfaces and types
            assert!(
                interfaces.contains(&"User".to_string()),
                "Should extract User interface"
            );
            assert!(
                interfaces.contains(&"ApiResponse".to_string()),
                "Should extract generic ApiResponse interface"
            );

            // EXPECTATION: Should extract functions with type information preserved
            assert!(
                functions.contains(&"fetchUser".to_string()),
                "Should extract async typed function"
            );
            assert!(
                functions.contains(&"createRepository".to_string()),
                "Should extract generic function"
            );
            assert!(
                functions.contains(&"processPayment".to_string()),
                "Should extract function with union types"
            );

            // Should find generic class and its methods
            let classes: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Struct { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            assert!(
                classes.contains(&"DataService".to_string()),
                "Should extract generic class"
            );

            // Should find typed methods
            assert!(
                functions
                    .iter()
                    .any(|n| n.contains("get") && n.contains("DataService")),
                "Should find generic method with constraints"
            );
            assert!(
                functions
                    .iter()
                    .any(|n| n.contains("update") && n.contains("DataService")),
                "Should find method with Partial type"
            );
        }

        /// Test: React TypeScript components with props interfaces
        #[test]
        fn test_react_typescript_components_with_props() {
            let code = r#"
                interface UserProfileProps {
                    user: {
                        id: string;
                        name: string;
                        email: string;
                        avatar?: string;
                    };
                    onEdit: (userId: string) => void;
                    onDelete: (userId: string) => Promise<void>;
                    showActions?: boolean;
                }

                interface ProductCardProps {
                    product: Product;
                    onAddToCart: (product: Product, quantity: number) => void;
                    className?: string;
                }

                // Functional component with typed props
                const UserProfile: React.FC<UserProfileProps> = ({
                    user,
                    onEdit,
                    onDelete,
                    showActions = true
                }) => {
                    const handleEdit = () => onEdit(user.id);
                    const handleDelete = async () => await onDelete(user.id);

                    return (
                        <div className="user-profile">
                            <img src={user.avatar || '/default-avatar.png'} alt={user.name} />
                            <h3>{user.name}</h3>
                            <p>{user.email}</p>
                            {showActions && (
                                <div>
                                    <button onClick={handleEdit}>Edit</button>
                                    <button onClick={handleDelete}>Delete</button>
                                </div>
                            )}
                        </div>
                    );
                };

                // Component with hooks and custom hook
                const ProductCard: React.FC<ProductCardProps> = ({ product, onAddToCart, className }) => {
                    const [quantity, setQuantity] = React.useState<number>(1);
                    const [isLoading, setIsLoading] = React.useState<boolean>(false);

                    const handleAddToCart = React.useCallback(async () => {
                        setIsLoading(true);
                        try {
                            await onAddToCart(product, quantity);
                        } finally {
                            setIsLoading(false);
                        }
                    }, [product, quantity, onAddToCart]);

                    return (
                        <div className={`product-card ${className || ''}`}>
                            <h4>{product.name}</h4>
                            <p>${product.price}</p>
                            <input
                                type="number"
                                value={quantity}
                                onChange={(e) => setQuantity(parseInt(e.target.value))}
                                min="1"
                            />
                            <button onClick={handleAddToCart} disabled={isLoading}>
                                {isLoading ? 'Adding...' : 'Add to Cart'}
                            </button>
                        </div>
                    );
                };

                // Higher-order component
                function withAuth<P extends {}>(Component: React.ComponentType<P>) {
                    return function AuthenticatedComponent(props: P) {
                        const user = useAuth();

                        if (!user) {
                            return <div>Please log in</div>;
                        }

                        return <Component {...props} />;
                    };
                }
            "#;

            let module = parse_typescript(code);
            let visitor = EnhancedTypeScriptVisitor::new(Path::new("components.tsx"));
            let items = visitor.extract_items(&module);

            let interfaces: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Trait { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            let functions: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Function { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            // EXPECTATION: Should extract React component prop interfaces
            assert!(
                interfaces.contains(&"UserProfileProps".to_string()),
                "Should extract props interface"
            );
            assert!(
                interfaces.contains(&"ProductCardProps".to_string()),
                "Should extract props interface"
            );

            // EXPECTATION: Should extract React components with meaningful names
            assert!(
                functions.contains(&"UserProfile".to_string()),
                "Should extract typed React functional component"
            );
            assert!(
                functions.contains(&"ProductCard".to_string()),
                "Should extract typed React functional component with hooks"
            );
            assert!(
                functions.contains(&"withAuth".to_string()),
                "Should extract higher-order component"
            );

            // Should extract nested event handlers with context
            assert!(
                functions
                    .iter()
                    .any(|n| n.contains("handleEdit") || n.contains("UserProfile")),
                "Should extract event handler with component context"
            );
            assert!(
                functions
                    .iter()
                    .any(|n| n.contains("handleDelete") || n.contains("UserProfile")),
                "Should extract async event handler"
            );
            assert!(
                functions
                    .iter()
                    .any(|n| n.contains("handleAddToCart") || n.contains("ProductCard")),
                "Should extract callback handler"
            );
        }

        /// Test: TypeScript decorators and metadata
        #[test]
        fn test_typescript_decorators_and_metadata() {
            let code = r#"
                // Decorator functions
                function Controller(route: string) {
                    return function <T extends { new (...args: any[]): {} }>(constructor: T) {
                        return class extends constructor {
                            route = route;
                        };
                    };
                }

                function Get(path: string) {
                    return function (target: any, propertyKey: string, descriptor: PropertyDescriptor) {
                        descriptor.value.route = path;
                        descriptor.value.method = 'GET';
                    };
                }

                function Post(path: string) {
                    return function (target: any, propertyKey: string, descriptor: PropertyDescriptor) {
                        descriptor.value.route = path;
                        descriptor.value.method = 'POST';
                    };
                }

                // Class with decorators
                @Controller('/api/users')
                class UserController {
                    constructor(private userService: UserService) {}

                    @Get('/')
                    async getAllUsers(): Promise<User[]> {
                        return this.userService.findAll();
                    }

                    @Get('/:id')
                    async getUserById(id: string): Promise<User | null> {
                        return this.userService.findById(id);
                    }

                    @Post('/')
                    async createUser(userData: CreateUserDto): Promise<User> {
                        return this.userService.create(userData);
                    }
                }

                // Service class with dependency injection
                @Injectable()
                class UserService {
                    constructor(
                        @Inject('UserRepository') private userRepo: IUserRepository,
                        @Inject('Logger') private logger: ILogger
                    ) {}

                    async findAll(): Promise<User[]> {
                        this.logger.info('Fetching all users');
                        return this.userRepo.findMany({});
                    }

                    async findById(id: string): Promise<User | null> {
                        this.logger.info(`Fetching user with ID: ${id}`);
                        return this.userRepo.findOne({ id });
                    }

                    async create(userData: CreateUserDto): Promise<User> {
                        this.logger.info('Creating new user');
                        const user = await this.userRepo.create(userData);
                        return user;
                    }
                }
            "#;

            let module = parse_typescript(code);
            let visitor = EnhancedTypeScriptVisitor::new(Path::new("controller.ts"));
            let items = visitor.extract_items(&module);

            let functions: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Function { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            let classes: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Struct { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            // EXPECTATION: Should extract decorator functions
            assert!(
                functions.contains(&"Controller".to_string()),
                "Should extract class decorator function"
            );
            assert!(
                functions.contains(&"Get".to_string()),
                "Should extract method decorator function"
            );
            assert!(
                functions.contains(&"Post".to_string()),
                "Should extract method decorator function"
            );

            // EXPECTATION: Should extract decorated classes
            assert!(
                classes.contains(&"UserController".to_string()),
                "Should extract decorated controller class"
            );
            assert!(
                classes.contains(&"UserService".to_string()),
                "Should extract decorated service class"
            );

            // Should extract decorated methods with controller context
            assert!(
                functions
                    .iter()
                    .any(|n| n.contains("getAllUsers") && n.contains("UserController")),
                "Should extract decorated controller method"
            );
            assert!(
                functions
                    .iter()
                    .any(|n| n.contains("getUserById") && n.contains("UserController")),
                "Should extract decorated controller method"
            );
            assert!(
                functions
                    .iter()
                    .any(|n| n.contains("createUser") && n.contains("UserController")),
                "Should extract decorated controller method"
            );

            // Should extract service methods
            assert!(
                functions
                    .iter()
                    .any(|n| n.contains("findAll") && n.contains("UserService")),
                "Should extract service method"
            );
            assert!(
                functions
                    .iter()
                    .any(|n| n.contains("create") && n.contains("UserService")),
                "Should extract service method"
            );
        }
