import pygame

class InputHandler:
    def get_input(self) -> tuple[int, bool]:
        """
        Retorna (direccion, salir):
        direccion: 0=quieto, 1=arriba, 2=abajo
        salir: True si el usuario cerró la ventana o presionó Escape
        """
        quit_requested = False
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                quit_requested = True
            elif event.type == pygame.KEYDOWN and event.key == pygame.K_ESCAPE:
                quit_requested = True

        keys = pygame.key.get_pressed()
        direction = 0
        if keys[pygame.K_UP] or keys[pygame.K_w]:
            direction = 1
        elif keys[pygame.K_DOWN] or keys[pygame.K_s]:
            direction = 2

        return direction, quit_requested
