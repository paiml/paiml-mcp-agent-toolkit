import { readFile } from 'fs/promises';
import * as path from 'path';

// Type definitions
interface User {
  id: number;
  name: string;
  email: string;
  isActive?: boolean;
}

type UserRole = 'admin' | 'user' | 'guest';

enum StatusCode {
  OK = 200,
  NOT_FOUND = 404,
  ERROR = 500,
}

// Class with methods and properties
export class UserService {
  private users: Map<number, User> = new Map();
  private readonly apiKey: string;

  constructor(apiKey: string) {
    this.apiKey = apiKey;
  }

  async getUser(id: number): Promise<User | null> {
    return this.users.get(id) || null;
  }

  createUser(name: string, email: string): User {
    const id = this.users.size + 1;
    const user: User = { id, name, email, isActive: true };
    this.users.set(id, user);
    return user;
  }

  private validateEmail(email: string): boolean {
    return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
  }
}

// Function declarations
export function processData(data: string[]): Record<string, number> {
  const counts: Record<string, number> = {};
  for (const item of data) {
    counts[item] = (counts[item] || 0) + 1;
  }
  return counts;
}

export async function fetchRemoteData(url: string): Promise<string> {
  // Simulated async operation
  return `Data from ${url}`;
}

// Arrow function exports
export const calculateSum = (numbers: number[]): number => {
  return numbers.reduce((acc, num) => acc + num, 0);
};

export const asyncOperation = async (value: string): Promise<string> => {
  await new Promise(resolve => setTimeout(resolve, 100));
  return value.toUpperCase();
};

// Interface extending another interface
interface AdminUser extends User {
  permissions: string[];
  role: UserRole;
}

// Type alias for complex type
type ApiResponse<T> = {
  data: T;
  status: StatusCode;
  timestamp: Date;
};

// Generic class
class Repository<T extends { id: number }> {
  private items: T[] = [];

  add(item: T): void {
    this.items.push(item);
  }

  findById(id: number): T | undefined {
    return this.items.find(item => item.id === id);
  }
}