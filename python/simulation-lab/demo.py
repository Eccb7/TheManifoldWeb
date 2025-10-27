"""
Mesa-based Agent Simulation for The Manifold Web

Demonstrates basic agent behavior: movement, energy consumption, and resource
gathering in a grid environment.
"""

import random
from mesa import Agent, Model
from mesa.time import RandomActivation
from mesa.space import MultiGrid
from mesa.datacollection import DataCollector


class Resource:
    """A resource in the environment that agents can consume."""

    def __init__(self, energy_value):
        self.energy_value = energy_value
        self.consumed = False


class ManifoldAgent(Agent):
    """An autonomous agent in the Manifold simulation."""

    def __init__(self, unique_id, model, initial_energy=100):
        super().__init__(unique_id, model)
        self.energy = initial_energy
        self.alive = True
        self.resources_collected = 0

    def move(self):
        """Move to a random neighboring cell."""
        possible_steps = self.model.grid.get_neighborhood(
            self.pos, moore=True, include_center=False
        )
        new_position = self.random.choice(possible_steps)
        self.model.grid.move_agent(self, new_position)

    def consume_resource(self):
        """Check for and consume resources at current position."""
        cellmates = self.model.grid.get_cell_list_contents([self.pos])
        resources = [obj for obj in cellmates if isinstance(obj, Resource)]

        for resource in resources:
            if not resource.consumed:
                self.energy += resource.energy_value
                resource.consumed = True
                self.resources_collected += 1
                print(
                    f"Agent {self.unique_id} consumed resource worth "
                    f"{resource.energy_value} energy (total: {self.energy})"
                )
                break

    def step(self):
        """Execute one time step of agent behavior."""
        if not self.alive:
            return

        # Move to new position
        self.move()

        # Lose energy from movement
        self.energy -= 1

        # Try to consume resource at new position
        self.consume_resource()

        # Check if agent dies from lack of energy
        if self.energy <= 0:
            self.alive = False
            print(f"Agent {self.unique_id} died from lack of energy")


class ManifoldModel(Model):
    """The main simulation model."""

    def __init__(
        self, n_agents=10, width=20, height=20, n_resources=15, seed=None
    ):
        super().__init__()
        self.num_agents = n_agents
        self.grid = MultiGrid(width, height, torus=True)
        self.schedule = RandomActivation(self)

        if seed is not None:
            random.seed(seed)

        # Create agents
        for i in range(self.num_agents):
            agent = ManifoldAgent(i, self, initial_energy=100)
            self.schedule.add(agent)

            # Place agent at random position
            x = self.random.randrange(self.grid.width)
            y = self.random.randrange(self.grid.height)
            self.grid.place_agent(agent, (x, y))

        # Create resources
        for _ in range(n_resources):
            resource = Resource(energy_value=10)
            x = self.random.randrange(self.grid.width)
            y = self.random.randrange(self.grid.height)
            self.grid.place_agent(resource, (x, y))

        # Data collection
        self.datacollector = DataCollector(
            model_reporters={
                "Alive Agents": lambda m: sum(
                    1 for a in m.schedule.agents if a.alive
                )
            },
            agent_reporters={"Energy": "energy", "Alive": "alive"},
        )

    def step(self):
        """Advance the model by one step."""
        self.datacollector.collect(self)
        self.schedule.step()


def run_simulation(steps=50, n_agents=10, seed=42):
    """Run the simulation and display results."""
    print("=" * 60)
    print("🌐 The Manifold Web - Agent Simulation")
    print("=" * 60)
    print(f"Initializing with {n_agents} agents...\n")

    model = ManifoldModel(n_agents=n_agents, seed=seed)

    print(f"Running simulation for {steps} steps...\n")

    for i in range(steps):
        model.step()
        if (i + 1) % 10 == 0:
            alive = sum(1 for a in model.schedule.agents if a.alive)
            print(f"Step {i + 1}: {alive} agents alive")

    print("\n" + "=" * 60)
    print("Simulation Complete")
    print("=" * 60)

    # Summary statistics
    alive_agents = [a for a in model.schedule.agents if a.alive]
    print(f"\nFinal Status:")
    print(f"  Alive agents: {len(alive_agents)}")
    print(f"  Dead agents: {model.num_agents - len(alive_agents)}")

    if alive_agents:
        avg_energy = sum(a.energy for a in alive_agents) / len(alive_agents)
        total_resources = sum(a.resources_collected for a in model.schedule.agents)
        print(f"  Average energy (alive): {avg_energy:.2f}")
        print(f"  Total resources collected: {total_resources}")

    print("\n" + "=" * 60)

    return model


if __name__ == "__main__":
    model = run_simulation(steps=50, n_agents=10, seed=42)
