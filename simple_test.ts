// Simple TypeScript test file
function regularFunction() {
    return "hello";
}

const arrowFunction = () => {
    return "world";
};

class TestClass {
    method() {
        return "class method";
    }

    static staticMethod() {
        return "static method";
    }
}

const objectWithMethods = {
    methodName: function() {
        return "object method";
    },
    asyncMethod: async function() {
        return "async object method";
    }
};