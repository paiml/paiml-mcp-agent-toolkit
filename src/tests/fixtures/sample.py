#!/usr/bin/env python3
"""Sample Python file for AST parsing tests."""

import os
import sys
from typing import List, Optional, Dict
from dataclasses import dataclass

# Global constant
API_VERSION = "1.0.0"


@dataclass
class User:
    """User model with basic attributes."""
    id: int
    name: str
    email: str
    is_active: bool = True
    
    def get_display_name(self) -> str:
        """Get formatted display name."""
        return f"{self.name} ({self.email})"
    
    def _internal_method(self):
        """Private internal method."""
        pass


class UserService:
    """Service for managing users."""
    
    def __init__(self):
        self.users: Dict[int, User] = {}
        self._cache = {}
    
    async def get_user(self, user_id: int) -> Optional[User]:
        """Retrieve a user by ID."""
        return self.users.get(user_id)
    
    async def create_user(self, name: str, email: str) -> User:
        """Create a new user."""
        user_id = len(self.users) + 1
        user = User(id=user_id, name=name, email=email)
        self.users[user_id] = user
        return user
    
    def list_users(self) -> List[User]:
        """List all users."""
        return list(self.users.values())


def process_data(data: List[str]) -> Dict[str, int]:
    """Process a list of strings and return counts."""
    counts = {}
    for item in data:
        counts[item] = counts.get(item, 0) + 1
    return counts


async def fetch_remote_data(url: str) -> str:
    """Fetch data from a remote URL."""
    # Simulated async operation
    return f"Data from {url}"


def _private_helper(value: int) -> int:
    """Private helper function."""
    return value * 2


# Module-level code
if __name__ == "__main__":
    service = UserService()
    print(f"API Version: {API_VERSION}")