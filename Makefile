generated_path := src/generated

generate_fbs:
	# Requires flatc complier
	flatc -r -o shared/src/generated shared/src/game_state.fbs shared/src/player_commands.fbs
