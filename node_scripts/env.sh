#!/bin/bash

unset DOCKER_REGISTRY
unset DOCKER_REPO
unset SC
unset RECOVER_NODE_SC
unset SHARE_SC
unset STS_NAME
unset BACKUP_NODE
unset RECOVER_NODE
unset ARGS

export DOCKER_REGISTRY=docker.io
export DOCKER_REPO=jayanring

export NAME_SPACE=chain-cache

export RECOVER_NODE_SC=nas-client-provisioner # RWO
export SHARE_SC=nas-client-provisioner # RWX

export STS_NAME=sla-overlord-node5