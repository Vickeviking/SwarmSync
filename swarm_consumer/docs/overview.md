# Swarm consumer

## Purpose

Swarm consumer is a cli program that acts as the user endpoint of the program.
Users can login as an authenticated user or register one through ´swarm_consumer´ to then upload jobs,
check on jobs aswell as fetch finnished jobs. Workers are not deployed here but instead seperately through the
´swarm-worker-tui´ inside the ´swarm-worker´ module.

## Pre-requisites

To run swarm-consumer one has to run a core service. This can be done either localy or on a remote server.
This can be configured inside the cli. Since the app is dockerized the only thing needed is a docker-daemon running.
Remember for jobs to be successfully executed a worker is needed, one is started through ´swarm-worker-tui` and
linked through the swarm-core's commanddeck cli.
