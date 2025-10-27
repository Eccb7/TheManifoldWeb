"""
Genetic Algorithm Demo using DEAP

Demonstrates evolutionary computation with byte-array genomes similar to the
Rust implementation in manifold-node.
"""

import random
from deap import base, creator, tools, algorithms
import numpy as np


def evaluate_genome(individual):
    """
    Fitness function: sum of byte values (simple example).

    In a real scenario, this would execute the genome's behavior and measure
    performance metrics like energy collected, survival time, or reproductive
    success.
    """
    return (sum(individual),)


def mutate_byte_flip(individual, mutation_rate):
    """
    Bit-flip mutation matching the Rust implementation.

    For each byte in the genome, flip random bits with probability
    mutation_rate.
    """
    for i in range(len(individual)):
        if random.random() < mutation_rate:
            # Flip a random bit in this byte
            bit_position = random.randint(0, 7)
            individual[i] ^= 1 << bit_position
    return (individual,)


def crossover_single_point(ind1, ind2):
    """
    Single-point crossover matching the Rust implementation.

    Selects a random crossover point and swaps segments between parents.
    """
    size = min(len(ind1), len(ind2))
    if size > 1:
        crossover_point = random.randint(1, size - 1)
        ind1[crossover_point:], ind2[crossover_point:] = (
            ind2[crossover_point:].copy(),
            ind1[crossover_point:].copy(),
        )
    return ind1, ind2


def create_toolbox(genome_length=32, mutation_rate=0.01):
    """Set up DEAP toolbox with genetic operators."""

    # Create fitness and individual types
    creator.create("FitnessMax", base.Fitness, weights=(1.0,))
    creator.create("Individual", list, fitness=creator.FitnessMax)

    toolbox = base.Toolbox()

    # Genome initialization: random bytes (0-255)
    toolbox.register("byte", random.randint, 0, 255)
    toolbox.register(
        "individual",
        tools.initRepeat,
        creator.Individual,
        toolbox.byte,
        genome_length,
    )
    toolbox.register("population", tools.initRepeat, list, toolbox.individual)

    # Genetic operators
    toolbox.register("evaluate", evaluate_genome)
    toolbox.register("mate", crossover_single_point)
    toolbox.register("mutate", mutate_byte_flip, mutation_rate=mutation_rate)
    toolbox.register("select", tools.selTournament, tournsize=3)

    return toolbox


def run_evolution(
    population_size=50,
    genome_length=32,
    generations=20,
    mutation_rate=0.01,
    crossover_rate=0.7,
):
    """Run the genetic algorithm and display results."""

    print("=" * 60)
    print("🧬 The Manifold Web - Genetic Algorithm Demo")
    print("=" * 60)
    print(f"Population size: {population_size}")
    print(f"Genome length: {genome_length} bytes")
    print(f"Generations: {generations}")
    print(f"Mutation rate: {mutation_rate}")
    print(f"Crossover rate: {crossover_rate}")
    print("=" * 60 + "\n")

    toolbox = create_toolbox(genome_length, mutation_rate)

    # Create initial population
    population = toolbox.population(n=population_size)

    # Statistics tracking
    stats = tools.Statistics(lambda ind: ind.fitness.values)
    stats.register("avg", np.mean)
    stats.register("std", np.std)
    stats.register("min", np.min)
    stats.register("max", np.max)

    # Run evolution
    print("Starting evolution...\n")

    population, logbook = algorithms.eaSimple(
        population,
        toolbox,
        cxpb=crossover_rate,
        mutpb=mutation_rate,
        ngen=generations,
        stats=stats,
        verbose=True,
    )

    # Results
    print("\n" + "=" * 60)
    print("Evolution Complete")
    print("=" * 60)

    best_individual = tools.selBest(population, k=1)[0]
    print(f"\nBest genome fitness: {best_individual.fitness.values[0]}")
    print(f"Best genome (first 16 bytes): {best_individual[:16]}")
    print(f"Best genome (hex): {' '.join(f'{b:02x}' for b in best_individual[:16])}")

    return population, logbook, best_individual


def demonstrate_crossover_and_mutation():
    """Demonstrate crossover and mutation operations."""

    print("\n" + "=" * 60)
    print("Crossover and Mutation Demo")
    print("=" * 60 + "\n")

    # Create two parent genomes
    parent1 = [0xFF] * 10
    parent2 = [0x00] * 10

    print("Parent 1:", " ".join(f"{b:02x}" for b in parent1))
    print("Parent 2:", " ".join(f"{b:02x}" for b in parent2))

    # Crossover
    child1, child2 = crossover_single_point(parent1.copy(), parent2.copy())
    print("\nAfter crossover:")
    print("Child 1: ", " ".join(f"{b:02x}" for b in child1))
    print("Child 2: ", " ".join(f"{b:02x}" for b in child2))

    # Mutation
    mutated = mutate_byte_flip(child1.copy(), mutation_rate=0.5)[0]
    print("\nAfter mutation (50% rate):")
    print("Mutated: ", " ".join(f"{b:02x}" for b in mutated))

    print("\n" + "=" * 60)


if __name__ == "__main__":
    # Demonstrate crossover and mutation
    demonstrate_crossover_and_mutation()

    # Run full evolution
    print("\n\n")
    population, logbook, best = run_evolution(
        population_size=50,
        genome_length=32,
        generations=20,
        mutation_rate=0.01,
        crossover_rate=0.7,
    )
