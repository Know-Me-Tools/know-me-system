# KnowMe Kubernetes deployment

The `base` Kustomization deploys the currently executable anonymous KnowMe Axum/React
server. It contains no provider credentials and expects an operator-built or registry-
published `knowme-web` image.

```bash
kubectl kustomize deploy/k8s/base
kubectl apply -k deploy/k8s/base
```

Override the image without editing the base:

```bash
cd deploy/k8s
kustomize edit set image knowme-web=registry.example/knowme-web@sha256:<digest>
```

Do not create authenticated/realtime overlays by copying the Compose defaults. Those
profiles require published, mutually compatible Flint Forge/Fabric/Gate images, external
secret references, Kratos/Keto migration jobs, and the v2 tenant-scoped realtime contract.
The integration plan intentionally keeps those overlays absent until their authorization,
resume, and revocation gates pass; an unverified manifest would advertise a security
boundary that does not yet exist.
