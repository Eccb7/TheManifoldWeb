"""Unit tests for the simulation lab."""

import pytest
from demo import ManifoldAgent, ManifoldModel, Resource


def test_agent_creation():
    """Test that agents are created with correct initial state."""
    model = ManifoldModel(n_agents=1, width=10, height=10, n_resources=0, seed=42)
    agent = model.schedule.agents[0]

    assert agent.energy == 100
    assert agent.alive is True
    assert agent.resources_collected == 0


def test_agent_movement():
    """Test that agents can move to neighboring cells."""
    model = ManifoldModel(n_agents=1, width=10, height=10, n_resources=0, seed=42)
    agent = model.schedule.agents[0]

    initial_pos = agent.pos
    agent.move()
    new_pos = agent.pos

    # Position should change (unless grid is size 1x1)
    assert initial_pos != new_pos or (model.grid.width == 1 and model.grid.height == 1)


def test_energy_decay():
    """Test that agents lose energy when moving."""
    model = ManifoldModel(n_agents=1, width=10, height=10, n_resources=0, seed=42)
    agent = model.schedule.agents[0]

    initial_energy = agent.energy
    agent.step()

    # Energy should decrease by 1 after step
    assert agent.energy == initial_energy - 1


def test_resource_consumption():
    """Test that agents can consume resources."""
    model = ManifoldModel(n_agents=1, width=10, height=10, n_resources=1, seed=42)
    agent = model.schedule.agents[0]

    # Place resource at agent's position
    resource = Resource(energy_value=10)
    model.grid.place_agent(resource, agent.pos)

    initial_energy = agent.energy
    agent.consume_resource()

    # Energy should increase by resource value
    assert agent.energy == initial_energy + 10
    assert agent.resources_collected == 1
    assert resource.consumed is True


def test_agent_death():
    """Test that agents die when energy reaches zero."""
    model = ManifoldModel(n_agents=1, width=10, height=10, n_resources=0, seed=42)
    agent = model.schedule.agents[0]

    # Drain energy
    agent.energy = 1
    agent.step()

    # Agent should be dead
    assert agent.alive is False
    assert agent.energy <= 0


def test_simulation_runs():
    """Test that the full simulation can run without errors."""
    model = ManifoldModel(n_agents=5, width=10, height=10, n_resources=10, seed=42)

    # Run for 10 steps
    for _ in range(10):
        model.step()

    # Model should still have agents
    assert len(model.schedule.agents) == 5


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
