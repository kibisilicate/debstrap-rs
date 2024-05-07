build:
	cargo build --release
	pandoc "$(CURDIR)"/docs/debstrap.8.md --read=markdown --write=man --standalone --output="$(CURDIR)"/docs/debstrap.8

install:
	mkdir --parents "$(DESTDIR)"/usr/bin
	cp "$(CURDIR)"/target/release/debstrap "$(DESTDIR)"/usr/bin/debstrap
	chown root:root "$(DESTDIR)"/usr/bin/debstrap
	chmod 0755 "$(DESTDIR)"/usr/bin/debstrap

uninstall:
	rm --force "$(DESTDIR)"/usr/bin/debstrap

clean:
	rm --recursive --force "$(CURDIR)"/target
	rm --force "$(CURDIR)"/docs/debstrap.8

