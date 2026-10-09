import pygame

class InputHandler:
    def direction(self):
        if not pygame.key.get_focused():
            return 0
        keys = pygame.key.get_pressed()
        up = keys[pygame.K_UP] or keys[pygame.K_w]
        down = keys[pygame.K_DOWN] or keys[pygame.K_s]
        return 0 if up == down else (1 if up else 2)
