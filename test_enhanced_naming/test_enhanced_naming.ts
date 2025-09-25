// Test TypeScript file for enhanced naming

// Arrow function
const formatUserInfo = (name: string, age: number): string => {
  return `Name: ${name}, Age: ${age}`;
};

// Regular function
function calculateDiscount(price: number, percentage: number): number {
  return price * (percentage / 100);
}

// Class with methods
class ProductService {
  constructor(private apiUrl: string) {}

  async getAllProducts(): Promise<Product[]> {
    const response = await fetch(`${this.apiUrl}/products`);
    return response.json();
  }

  static validateProduct(product: Product): boolean {
    return product.name.length > 0 && product.price > 0;
  }
}

// Factory function with object methods
const createApiClient = (baseUrl: string) => {
  return {
    get: async (endpoint: string) => {
      return fetch(`${baseUrl}/${endpoint}`);
    },
    post: async (endpoint: string, data: any) => {
      return fetch(`${baseUrl}/${endpoint}`, {
        method: 'POST',
        body: JSON.stringify(data)
      });
    }
  };
};

// Interface
interface Product {
  id: number;
  name: string;
  price: number;
}

// Enum
enum Status {
  PENDING,
  COMPLETED,
  FAILED
}

export { formatUserInfo, calculateDiscount, ProductService, createApiClient, Product, Status };