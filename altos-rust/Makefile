binary = altos_rust
static_lib = lib$(binary).a
linker_script = cortex_m0.ld
target = thumbv6m-none-eabi
core_lib = altos_core
port_lib = cortex_m0

build_path = build/

debug_static_lib = target/$(target)/debug/$(static_lib)
debug_build_path = $(build_path)debug/
debug_build = $(debug_build_path)$(binary)

release_static_lib = target/$(target)/release/$(static_lib)
release_build_path = $(build_path)release/
release_build = $(release_build_path)$(binary)

doc_target_path = target/doc
doc_build_path = $(build_path)doc

path_to_cm0 = port/cortex-m0/

### CARGO ###
cargo = xargo
cargo_args = --target $(target)

### TEST ###
test_dependencies = altos_core \
					arm \
					volatile \
					cm0_atomic \
					cortex_m0 \
					bump_allocator \
					free_list_allocator \

# --lib flag only runs the unit test suite, doc tests are currently an issue for cross-compiled
#  platforms. See: https://github.com/rust-lang/cargo/issues/1789
test_args = $(foreach dep, $(test_dependencies),-p $(dep)) --lib

### DOC ###
doc_args = -p $(port_lib) --lib

### LINKER ###
linker = arm-none-eabi-ld
linker_args = -n --gc-sections -T $(linker_script)

### SIZE ###
size = arm-none-eabi-size
size_flags = -t

### GDB ###
gdb = arm-none-eabi-gdb
gdb_flags =
st_port = 4242
ocd_port = 3333
st_gdb_flags = $(gdb_flags) -eval-command="target remote :$(st_port)"
ocd_gdb_flags = $(gdb_flags) -eval-command="target remote :$(ocd_port)"

### Make targets ###

.PHONY: debug release clean

all: release

clean:
	@$(cargo) clean
	@rm -rf $(build_path)

debug: $(linker_script)
	@mkdir -p $(debug_build_path)
	@$(cargo) build $(cargo_args)
	@$(linker) $(linker_args) -o $(debug_build) $(debug_static_lib)
	@$(size) $(size_flags) $(debug_build)

release: $(linker_script)
	@mkdir -p $(release_build_path)
	@$(cargo) build $(cargo_args) --release
	@$(linker) $(linker_args) -o $(release_build) $(release_static_lib)
	@$(size) $(size_flags) $(release_build)

gdb: debug
	@$(gdb) $(gdb_flags) $(debug_build)

gdb-st: release
	@$(gdb) $(st_gdb_flags) $(release_build)

gdb-ocd: debug
	@$(gdb) $(ocd_gdb_flags) $(debug_build)

test:
	@cargo test $(test_args)

doc:
	@mkdir -p $(build_path)
	@cd port/cortex-m0; \
	cargo doc $(doc_args) --features "doc serial";
	@cp -r $(doc_target_path) $(doc_build_path);

test_verbose:
	@$(cargo) test $(test_args) -- --nocapture

size: debug
