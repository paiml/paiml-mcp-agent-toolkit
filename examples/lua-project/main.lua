--- Main entry point for the example Lua game
-- Demonstrates pmat analysis on a multi-file Lua project

local ok, Game = pcall(require, "game")
if not ok then error("Failed to load game module: " .. tostring(Game)) end
local ok2, Utils = pcall(require, "utils")
if not ok2 then error("Failed to load utils module: " .. tostring(Utils)) end

--- Create a player entity
local function create_player(x, y)
    local player = {
        x = x,
        y = y,
        radius = 16,
        health = 100,
        speed = 200,
        score = 0,
    }

    function player:update(dt)
        -- Simple AI: move toward center
        local target_x, target_y = 400, 300
        local dx = target_x - self.x
        local dy = target_y - self.y
        local dist = math.sqrt(dx * dx + dy * dy)

        if dist > 1 then
            self.x = self.x + (dx / dist) * self.speed * dt
            self.y = self.y + (dy / dist) * self.speed * dt
        end
    end

    function player:on_collision(other)
        if other.damage then
            self.health = self.health - other.damage
        end
    end

    return player
end

--- Create an enemy entity
local function create_enemy(x, y)
    local enemy = {
        x = x,
        y = y,
        radius = 12,
        health = 50,
        damage = 10,
        speed = 100,
    }

    function enemy:update(dt)
        -- Patrol pattern
        self.x = self.x + math.sin(os.clock() * 2) * self.speed * dt
        self.y = self.y + math.cos(os.clock() * 3) * self.speed * dt
    end

    function enemy:on_collision(other)
        if other.damage then
            self.health = self.health - other.damage
        end
    end

    return enemy
end

-- Main
local function main()
    print("=== Lua Game Demo ===")

    local game = Game:new(800, 600)
    local player = create_player(400, 300)
    game:add_entity(player)

    -- Spawn enemies
    for i = 1, 5 do
        local x = math.random(0, 800)
        local y = math.random(0, 600)
        game:add_entity(create_enemy(x, y))
    end

    -- Run for 60 frames
    game:run(60)

    -- Print results
    local state = game:serialize()
    print("Final state: " .. state)
    print("Player health: " .. player.health)

    -- Utils demo
    local numbers = { 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 }
    local evens = Utils.filter(numbers, function(n)
        return n % 2 == 0
    end)
    local sum = Utils.reduce(evens, function(acc, n)
        return acc + n
    end, 0)
    print("Sum of evens: " .. sum)
end

main()
