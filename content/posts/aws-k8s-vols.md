---
title: "Aws K8s Vols"
date: 2023-08-03T09:45:01+01:00
draft: true
---



storage class vs persistent volume  vs pvc
https://stackoverflow.com/questions/71389026/relationship-between-storageclass-persistent-volume-and-persistent-volume-claim

types of volume - ebs, efs, XFS (redpanda)

drivers
ebs (block storage):
https://github.com/kubernetes-sigs/aws-ebs-csi-driver/blob/master/docs/parameters.md

efs (filesystem):
https://github.com/kubernetes-sigs/aws-efs-csi-driver/tree/master

mount options https://man7.org/linux/man-pages/man5/ext4.5.html#MOUNT_OPTIONS


eks drivers, how do they work

static vs dynamic provisioning

https://github.com/kubernetes-sigs/aws-ebs-csi-driver/issues/1547

https://github.com/kubernetes-sigs/aws-ebs-csi-driver/issues/1365

https://github.com/kubernetes-sigs/aws-ebs-csi-driver/tree/master/examples/kubernetes/static-provisioning/manifests

what is kubernetes csi? https://github.com/kubernetes-csi
https://github.com/container-storage-interface/spec/blob/master/spec.md

https://snyk.io/blog/10-kubernetes-security-context-settings-you-should-understand/
