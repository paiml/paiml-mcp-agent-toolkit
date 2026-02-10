--- Game module: A simple game engine demonstrating Lua patterns
-- Used by pmat for Lua AST analysis demonstration

local json = require("dkjson")

local Game = {}
Game.__index = Game

--- Create a new game instance
-- @param width number Screen width
-- @param height number Screen height
-- @return Game
function Game.new(width, height)
    local self = setmetatable({}, Game)
    self.width = width or 800
    self.height = height or 600
    self.entities = {}
    self.running = false
    self.frame = 0
    return self
end

--- Add an entity to the game world
-- @param entity table Entity with x, y, update, draw methods
function Game:add_entity(entity)
    if not entity.x or not entity.y then
        error("Entity must have x and y coordinates")
    end
    table.insert(self.entities, entity)
end

--- Remove dead entities from the world
function Game:cleanup()
    local alive = {}
    for _, entity in ipairs(self.entities) do
        if entity.health and entity.health > 0 then
            table.insert(alive, entity)
        elseif not entity.health then
            table.insert(alive, entity)
        end
    end
    self.entities = alive
end

--- Update all entities
-- @param dt number Delta time in seconds
function Game:update(dt)
    for _, entity in ipairs(self.entities) do
        if entity.update then
            entity:update(dt)
        end

        -- Boundary check
        if entity.x < 0 then
            entity.x = 0
        elseif entity.x > self.width then
            entity.x = self.width
        end

        if entity.y < 0 then
            entity.y = 0
        elseif entity.y > self.height then
            entity.y = self.height
        end
    end

    self:cleanup()
    self.frame = self.frame + 1
end

--- Check collision between two entities
-- @param a table First entity
-- @param b table Second entity
-- @return boolean
function Game.check_collision(a, b)
    if not a.radius or not b.radius then
        return false
    end

    local dx = a.x - b.x
    local dy = a.y - b.y
    local distance = math.sqrt(dx * dx + dy * dy)
    return distance < (a.radius + b.radius)
end

--- Process all collisions in the world
function Game:process_collisions()
    local n = #self.entities
    for i = 1, n do
        for j = i + 1, n do
            local a = self.entities[i]
            local b = self.entities[j]
            if Game.check_collision(a, b) then
                if a.on_collision then
                    a:on_collision(b)
                end
                if b.on_collision then
                    b:on_collision(a)
                end
            end
        end
    end
end

--- Serialize game state to JSON
-- @return string JSON representation
function Game:serialize()
    local state = {
        width = self.width,
        height = self.height,
        frame = self.frame,
        entity_count = #self.entities,
    }
    return json.encode(state)
end

--- Run the game loop
-- @param max_frames number Maximum frames to run (nil for infinite)
function Game:run(max_frames)
    self.running = true
    local dt = 1 / 60

    while self.running do
        self:update(dt)
        self:process_collisions()

        if max_frames and self.frame >= max_frames then
            self.running = false
        end
    end
end

return Game
