# Concepts

## Note

Core concepts are described with a flake workflow in mind. But Typhon also
supports a more traditional workflow, see at the end of this section for
details.

## Overview

Typhon manages projects. A project typically corresponds to a repository and
the project's declaration is a Nix expression specifying the intended CI
workflow for that repository. A project defines jobsets, which generally track
branches. Jobsets are then periodically evaluated to produce jobs and
deployments, each evaluation being associated with a commit on that branch.

## Sources

Typhon's "sources" are values poiting to remote sources of Nix expressions.
They are encoded in JSON and must be "locked" before being evaluated. Locking a
source ensures that it can be evaluated reproducibly. For instance, if a source
points to a Git branch, locking it will produce a pointer to the head commit at
that time.  reproducibly.

## Projects

A project typically configures CI for a repository, but the declaration can
exist in a separate repository. In fact, the declaration of a project is quite
sensitive since it defines the way the project's unencrypted secrets are
handled. Malicious edits to the declaration can potentially leak these secrets.

## Jobsets

TODO

## Evaluations

TODO

## Jobs

TODO

## Actions

Actions are scripts run by Typhon in isolation from the system, but connected to
the internet. They are declared and built at the project level, then executed
through various hooks.

TODO
