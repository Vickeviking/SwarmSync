- Consumer
  A node that consumes power of the system, pushing up jobs to the core,
  `swarm-consumer` is the consumer model.
- Worker / Producer
  A node producing power and working on jobs,
  `swarm-worker` is responsible for this and can be used to spin up workers
- Dispatcher
  Handles a UDP server, allowing for dynmamic updates of worker availability.
  Sends out job that are ready to be run from `Scheduler` to the correct worker.
  Chooses worker, not job. Before sending the job to the worker it also updates
  `Harvester` on what jobs are being sent so it knows what to await and from whom.
- Harvester
  Recieves results from workers, when everything is fetched and deemed OK,
  it sends the result to `TaskArchive`.
- Hibernator
  After `Reciever` recieves a job from Consumer, it is either sent to `Scheduler` or to `Hibernator`.
  If it is not meant to be ran now, but instead holds a cron expression, it is sent to `Hibernator`.
  Here it sleeps for the correct time and then sends the job to `Scheduler`.
- Logger
  A struct protected by a mutex, shared between all of the modules.
  When modules need to log something they can log through the logger.
  Logger provides a rich interface with tags, filters and custom logging levels.
- Reciever
  Recieves jobs from the consumer, and sends them to the correct module.
- Scheduler
  Using a non-preemptive scheduling algorithm, it picks the next job to be sent to a worker.
  `Dispatcher` reads this job when its ready and sends it to the worker.
- TaskArchive
  A module that keeps track of the results of the jobs.
  Consumer can fetch the results from this module.
  In Future this will be deprecated, this since the meaning of this module
  seems to be unclear since we already have a postgres database for storing the results.
