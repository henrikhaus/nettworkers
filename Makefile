generated_path := src/generated

generate_fbs:
	# Requires flatc complier
	flatc -r -o src/generated src/game_state.fbs src/player_commands.fbs
