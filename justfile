rootdir := ''
etcdir := rootdir + '/etc'
prefix := rootdir + '/usr'
clean := '0'
debug := '0'
vendor := '0'
target := if debug == '1' { 'debug' } else { 'release' }
vendor_args := if vendor == '1' { '--frozen --offline' } else { '' }
debug_args := if debug == '1' { '' } else { '--release' }
cargo_args := vendor_args + ' ' + debug_args

dbusdir := etcdir + '/dbus-1/system.d'
bindir := prefix + '/bin'
systemddir := prefix + '/lib/systemd'

daemon_id := 'com.system76.PopSnapshot'
service_name := "pop-snapshot"

all: _extract_vendor
	cargo build {{cargo_args}}

# Installs files into the system
install:
	# dbus config, so root can host the daemon, and so we can talk to it without root
	install -Dm0644 service/data/{{daemon_id}}.xml {{dbusdir}}/{{daemon_id}}.conf

	# systemd service
	install -Dm0644 service/data/{{service_name}}.service {{systemddir}}/system/{{service_name}}.service

	# config file
	install -Dm0644 service/data/{{service_name}}.toml {{etcdir}}/{{service_name}}.toml

	# daemon
	install -Dm0755 target/release/pop-snapshot-daemon {{bindir}}/pop-snapshot-daemon

	# cli
	install -Dm0755 target/release/pop-snapshot {{bindir}}/pop-snapshot

	# cli completions
	install -Dm0644 target/pop-snapshot.bash {{etcdir}}/bash_completion.d/pop-snapshot
	install -Dm0644 target/_pop-snapshot {{prefix}}/share/zsh/vendor-completions/_pop-snapshot
	install -Dm0644 target/pop-snapshot.fish {{prefix}}/share/fish/completions/pop-snapshot.fish

clean_vendor:
	rm -rf vendor vendor.tar .cargo/config

clean: clean_vendor
	cargo clean

# Extracts vendored dependencies if vendor=1
_extract_vendor:
	#!/usr/bin/env sh
	if test {{vendor}} = 1; then
		rm -rf vendor; tar pxf vendor.tar
	fi
