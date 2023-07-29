---
title: "Helm: Nil Pointer Error Accessing Global Variables in a Range Loop"
date: 2023-07-03T15:12:19+01:00
summary: "A short lesson in Helm variable scoping."
draft: false
---

# TLDR

To reference global variables within a `range` block, prefix them with `$` which always points to the root (global) context, e.g. `$.Values.ingress.ports`.

# Overview

Recently I ran into an annoying problem whilst trying to fix a helm template, which left me stumped for a while. I was trying to reference a global variable in a `range` loop and running into an error:

```
nil pointer evaluating interface {}
```

I was modifying the following Helm template to create an [istio gateway](https://www.google.com/search?channel=fs&client=ubuntu&q=istio+gateway). The template defines a load balancer that proxies connections to downstream servers.

The template loops through an array of ports defined in the `.Values.service.ports` global variable, and defines a `port` for each of them, it then defines a single `hosts` entry:

```
apiVersion: networking.istio.io/v1alpha3
kind: Gateway
spec:
  servers:
  {{- range $name, $config := .Values.service.ports }}
    - port:
        number: {{ $config.ingress.port }}
        name: {{ $config.ingress.name }}
        protocol: {{ $config.ingress.protocol }}
  {{- end }}
      hosts:
      {{- range .Values.ingress.hosts }}
        - {{ include "service.host" . }}
      {{- end }}
```

I wanted to change it so that for each port, the `hosts` entry is repeated. To do this I did the obvious which was to modify the template to move the `hosts` entry inside the ports `range` loop:

```
apiVersion: networking.istio.io/v1alpha3
kind: Gateway
spec:
  servers:
  {{- range $name, $config := .Values.service.ports }}
    - port:
        number: {{ $config.ingress.port }}
        name: {{ $config.ingress.name }}
        protocol: {{ $config.ingress.protocol }}
      hosts:
      {{- range .Values.ingress.hosts }}
        - {{ include "service.host" . }}
      {{- end }}
  {{- end }}
```

Seems simple right?

However this started throwing an error which left me stumped for an upsettingly long time:

```bash
Error: template: app/templates/routing/gateway.yaml:21:23: 
executing "app/templates/routing/gateway.yaml" at <.Values.ingress.hosts>: 
nil pointer evaluating interface {}.ingress
```

I couldn't understand why the `.Values.ingress.hosts` was valid outside of the range loop but invalid inside it.

# The Solution - Helm Variable Scoping

This came down to understanding variable scoping which is kind of explained [here](https://helm.sh/docs/chart_template_guide/variables/) but not for this specific error:

### Variables in Helm templates are scoped to the block they are in. 

When the `.Values.ingress.hosts` was outside of the `range` block, `.Values` was valid, because
`.Values` is defined in the root/global scope and that is where it is being accessed.

However once `.Values.ingress.hosts` was moved into the `hosts` range block, there is no `.Values` defined in the local scope in the block.

**To fix this, `.Values` must be prefixed with `$` which always points to the root (global) context so it can be accessed with a `range` block**:

```
apiVersion: networking.istio.io/v1alpha3
kind: Gateway
spec:
  servers:
  {{- range $name, $config := .Values.service.ports }}
    - port:
        number: {{ $config.ingress.port }}
        name: {{ $config.ingress.name }}
        protocol: {{ $config.ingress.protocol }}
      hosts:
      {{- range $.Values.ingress.hosts }}
        - {{ include "service.host" . }}
      {{- end }}
  {{- end }}
```
