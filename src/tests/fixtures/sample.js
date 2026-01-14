// JavaScript sample file for AST parsing tests
const fs = require('fs');
const path = require('path');

// Class definition
class DataProcessor {
  constructor(options = {}) {
    this.options = options;
    this.data = [];
  }

  async loadData(filePath) {
    const content = await fs.promises.readFile(filePath, 'utf8');
    this.data = JSON.parse(content);
    return this.data;
  }

  processItems() {
    return this.data.map(item => ({
      ...item,
      processed: true,
      timestamp: new Date().toISOString()
    }));
  }
}

// Function declaration
function calculateAverage(numbers) {
  if (numbers.length === 0) return 0;
  const sum = numbers.reduce((acc, num) => acc + num, 0);
  return sum / numbers.length;
}

// Async function
async function fetchUserData(userId) {
  // Simulated API call
  return new Promise((resolve) => {
    setTimeout(() => {
      resolve({
        id: userId,
        name: 'John Doe',
        email: 'john@example.com'
      });
    }, 100);
  });
}

// Arrow function stored in const
const formatDate = (date) => {
  return new Intl.DateTimeFormat('en-US').format(date);
};

// Function expression
const validateInput = function(input) {
  return input && typeof input === 'string' && input.length > 0;
};

// Async arrow function
const processAsync = async (data) => {
  const results = await Promise.all(
    data.map(async (item) => {
      return await transformItem(item);
    })
  );
  return results;
};

// Module exports
module.exports = {
  DataProcessor,
  calculateAverage,
  fetchUserData,
  formatDate,
  validateInput,
  processAsync
};

// Export individual items
exports.VERSION = '1.0.0';
exports.DEFAULT_TIMEOUT = 5000;