---
title: "How to use bash functions in docker-compose yaml files"
date: 2019-07-24T21:51:08+01:00
summary: "There is no supported way of evaluating bash functions in Docker Compose files, this post describes a workaround."
draft: false
---

Docker Compose supports setting environment variables in `docker-compose.yml` files, you just need to create a `.env` file and then reference variables in the docker-compose.yml file using `${variable}` syntax.

However there is no supported way of evaluating bash functions in these `.env` files, or the `docker-compose.yml` files.

There are plenty of cases where it is useful to be able to do that though, e.g. to obtain a value from `aws cli`.

# Example: How to set variables using the supported `.env` file method

Create a `.env` file with a variable:
{{< highlight bash >}}
SRC=~/code/app
{{< / highlight >}}

You can now reference the `SRC` variable in a `docker-compose.yml` file:
{{< highlight yaml >}}
version: '3.7'
services:
  app:
    image: "app"
    volumes:
      - ${SRC}:/mnt/src
{{< / highlight >}}

Spin up the service using `docker-compose up` and the `.env` file is read automagically.

It is also possible to specify alternative env file names using the `env_file` key in the `docker-compose.yml` file, e.g.:

{{< highlight yaml >}}
version: '3.7'
services:
  app:
    image: "app"
    env_file: ~/code/app/app.env
{{< / highlight >}}

However the `.env` file is not a bash script, so we can't run bash functions there.

# Solution

The way to get around this is to create an executable bash script.

In this bash script you can call bash functions and assign their output to variables, and with some additional bash wizadry you can access these in the `docker-compose.yml` file in the same way.

Create an executable bash file (e.g. `env.sh`):
{{< highlight bash >}}
#/bin/bash
AWS_ACCESS_KEY_ID=$(aws configure get aws_access_key_id)
{{< / highlight >}}

Now reference `AWS_ACCESS_KEY_ID` in the `docker-compose.yml` file:
{{< highlight yaml >}}
version: '3.7'
services:
  app:
    image: "app"
    environment:
      - AWS_ACCESS_KEY_ID=${AWS_ACCESS_KEY_ID}
{{< / highlight >}}

We are not able to specify the executable env file in the `env_file` key, but if we run the following commands, the variables set in `env.sh` will be available in the `docker-compose.yml` file:

{{< highlight bash >}}
set -a; source env.sh; set +a; docker-compose up app
{{< / highlight >}}

`set -a` is short for `set allexport`. This command will mark the variables created when we run `source env.sh` for export, making them available to other processes that might run in this shell. They are now going to be available when we run `docker-compose up`. Lovely.

`set +a` is just the opposite of `set -a`, so it will disable the exporting behaviour enabled with `set -a`. This is for safety, in case we forget about this and run further commands in the shell later which may overwrite other shell variables without us realising.

These commands will only create shell variables (i.e. they're only available in the current shell) so there's no need to worry about this overwriting variables in your global environment.

If we wanted to do this in a bash script running `docker-compose up` then the following example works:

{{< highlight bash >}}
#/bin/bash
set -a
. /env.sh
set +a

docker-compose up app
{{< / highlight >}}

`. /env.sh` is equivalent to `source /env.sh` but `.` is available in all Bourne-like shells (whereas `source` is not always available), so this makes the script more portable too.
