#!/bin/bash

unset DOCKER_REGISTRY
unset DOCKER_REPO
unset SC
unset NEW_NODE_SC
unset SHARE_SC
unset STS_NAME

export DOCKER_REGISTRY=docker.io
export DOCKER_REPO=jayanring

export NEW_NODE_SC=csi-disk # RWO
export SHARE_SC=csi-nas # RWX
export NAME_SPACE=chain-cache

export STS_NAME=sla-overlord-node5