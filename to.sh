set -ex

IMG=quay.io/fdeutsch/deltasched

build() {
podman -r build -t $IMG -f Containerfile .
}

run() {
podman -r run --rm -p 8000:8000 -v schedule/:/app/schedule $IMG
}

scheds_cm() {
kubectl create configmap --dry-run=client --from-file=schedule/streams.json --from-file=schedule/4.15.yaml --from-file=schedule/4.16.yaml --from-file=schedule/4.17.yaml scheds -o yaml
}

eval $@
