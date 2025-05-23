generated_path := src/generated

generate_fbs:
	# Requires flatc complier
	flatc --rust
