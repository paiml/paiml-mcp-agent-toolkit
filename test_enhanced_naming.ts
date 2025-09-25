// Test TypeScript file for enhanced naming
import React from 'react';

// React Component (with JSX removed for parsing compatibility)
const UserProfile: React.FC<{name: string, age: number}> = ({ name, age }) => {
  return `<div><h1>${name}</h1><p>Age: ${age}</p></div>`;
};

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

export { UserProfile, ProductService, createApiClient, Product, Status };