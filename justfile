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
	install -Dm0644 service/data/{{daemon_id}}.xml {{dbusdir}}/{{daemon_id}}.xml

	# systemd service
	install -Dm0644 service/data/{{service_name}}.service {{systemddir}}/system/{{service_name}}.service

	# daemon
	install -Dm0755 target/release/pop-snapshot-daemon {{bindir}}/pop-snapshot-daemon

	# cli
	install -Dm0755 target/release/pop-snapshot {{bindir}}/pop-snapshot

# Extracts vendored dependencies if vendor=1
_extract_vendor:
	#!/usr/bin/env sh
	if test {{vendor}} = 1; then
		rm -rf vendor; tar pxf vendor.tar
	fi
