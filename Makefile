all:
	docker build --target client -t zkp-poc-client .
	docker build --target server -t zkp-poc-server .
