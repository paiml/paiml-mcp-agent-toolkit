--- Utility functions for the game engine
-- Demonstrates various Lua patterns for pmat analysis

local Utils = {}

--- Clamp a value between min and max
-- @param value number
-- @param min number
-- @param max number
-- @return number
function Utils.clamp(value, min, max)
    if value < min then
        return min
    elseif value > max then
        return max
    end
    return value
end

--- Linear interpolation
-- @param a number Start value
-- @param b number End value
-- @param t number Interpolation factor (0-1)
-- @return number
function Utils.lerp(a, b, t)
    return a + (b - a) * Utils.clamp(t, 0, 1)
end

--- Deep copy a table
-- @param original table
-- @return table
function Utils.deep_copy(original)
    if type(original) ~= "table" then
        return original
    end

    local copy = {}
    for key, value in pairs(original) do
        if type(value) == "table" then
            copy[key] = Utils.deep_copy(value)
        else
            copy[key] = value
        end
    end

    return setmetatable(copy, getmetatable(original))
end

--- Merge two tables (shallow)
-- @param base table
-- @param override table
-- @return table
function Utils.merge(base, override)
    local result = {}
    for k, v in pairs(base) do
        result[k] = v
    end
    for k, v in pairs(override) do
        result[k] = v
    end
    return result
end

--- Filter a list based on a predicate
-- @param list table Array-like table
-- @param predicate function Returns true to keep element
-- @return table Filtered list
function Utils.filter(list, predicate)
    local result = {}
    for _, item in ipairs(list) do
        if predicate(item) then
            table.insert(result, item)
        end
    end
    return result
end

--- Map a function over a list
-- @param list table Array-like table
-- @param transform function Transform each element
-- @return table Mapped list
function Utils.map(list, transform)
    local result = {}
    for i, item in ipairs(list) do
        result[i] = transform(item)
    end
    return result
end

--- Reduce a list to a single value
-- @param list table Array-like table
-- @param reducer function(accumulator, item) -> accumulator
-- @param initial any Initial accumulator value
-- @return any
function Utils.reduce(list, reducer, initial)
    local acc = initial
    for _, item in ipairs(list) do
        acc = reducer(acc, item)
    end
    return acc
end

--- Create a simple event emitter
-- @return table Event emitter with on/emit/off methods
function Utils.create_emitter()
    local emitter = { _handlers = {} }

    function emitter:on(event, handler)
        if not self._handlers[event] then
            self._handlers[event] = {}
        end
        table.insert(self._handlers[event], handler)
    end

    function emitter:emit(event, ...)
        local handlers = self._handlers[event]
        if handlers then
            for _, handler in ipairs(handlers) do
                handler(...)
            end
        end
    end

    function emitter:off(event)
        self._handlers[event] = nil
    end

    return emitter
end

return Utils
