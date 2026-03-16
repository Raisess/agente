# Tic Tac Toe Game

def print_board(board):
    for row in board:
        print(' | '.join(row))
        print('-' * 5)


def check_winner(board, player):
    for row in board:
        if all([cell == player for cell in row]):
            return True
    for col in range(3):
        if all([board[row][col] == player for row in range(3)]):
            return True
    if all([board[i][i] == player for i in range(3)]) or all([board[i][2-i] == player for i in range(3)]):
        return True
    return False


def tic_tac_toe():
    board = [[' ']*3 for _ in range(3)]
    player = 'X'
    print_board(board)
    for _ in range(9):
        while True:
            try:
                row, col = map(int, input(f'Player {player}, enter row and column (0-2): ').split())
                if row not in range(3) or col not in range(3):
                    raise ValueError('Row or column value out of range')
                if board[row][col] != ' ':
                    print('Invalid move! Try again.')
                    continue
                board[row][col] = player
                print_board(board)
                if check_winner(board, player):
                    print(f'Player {player} wins!')
                    break
                player = 'O' if player == 'X' else 'X'
                break
            except ValueError as ve:
                print(ve)

    else:
        print('It is a tie!')


if __name__ == '__main__':
    tic_tac_toe()
