import numpy as np
import pygame

# Constants
WIDTH, HEIGHT = 800, 600
CELL_SIZE = 10
ROWS, COLS = HEIGHT // CELL_SIZE, WIDTH // CELL_SIZE
BLACK = (0, 0, 0)
WHITE = (255, 255, 255)


def initialize_grid():
    return np.random.choice([0, 1], (ROWS, COLS), p=[0.9, 0.1])


def get_neighbors(grid, x, y):
    sum = 0
    for i in range(-1, 2):
        for j in range(-1, 2):
            row = (x + i + ROWS) % ROWS
            col = (y + j + COLS) % COLS
            sum += grid[row][col]
    sum -= grid[x][y]
    return sum


def update_grid(grid):
    new_grid = grid.copy()
    for i in range(ROWS):
        for j in range(COLS):
            neighbors = get_neighbors(grid, i, j)
            if grid[i][j] == 1:
                if neighbors < 2 or neighbors > 3:
                    new_grid[i][j] = 0
            else:
                if neighbors == 3:
                    new_grid[i][j] = 1

    return new_grid


def draw_grid(screen, grid):
    for i in range(ROWS):
        for j in range(COLS):
            color = WHITE if grid[i][j] == 1 else BLACK
            pygame.draw.rect(screen, color, (j * CELL_SIZE,
                             i * CELL_SIZE, CELL_SIZE, CELL_SIZE))


def main():
    pygame.init()
    screen = pygame.display.set_mode((WIDTH, HEIGHT))
    pygame.display.set_caption("Conway's Game of Life")

    clock = pygame.time.Clock()
    grid = initialize_grid()

    running = True
    while running:
        screen.fill(BLACK)

        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                running = False

        draw_grid(screen, grid)
        grid = update_grid(grid)

        pygame.display.flip()
        clock.tick(10)  # Adjust speed by changing ticks per second

    pygame.quit()


if __name__ == "__main__":
    main()
