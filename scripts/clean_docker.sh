#!/bin/bash

set -e  # Exit on error

echo "Pruning all Docker images, containers, networks, and volumes..."
docker system prune -a --volumes -f

