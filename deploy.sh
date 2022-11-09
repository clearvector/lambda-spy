#!/bin/bash

usage() { echo "Usage: $0 [-p <aws_profile>] [-r <aws_region>]" 1>&2; exit 1; }

while getopts ":p:r:" o; do
    case "${o}" in
        p)
            p=${OPTARG}
            AWS_PROFILE="${p}"
            ;;
        r)
            r=${OPTARG}
            AWS_REGION="${r}"
            ;;
        *)
            usage
            ;;
    esac
done
shift $((OPTIND-1))

if [ -z "${AWS_PROFILE}" ]; then
    echo "No AWS profile supplied"
    usage
fi

if [ -z "${AWS_REGION}" ]; then
    echo "No AWS region supplied"
    usage
fi

BASENAME="$(basename $(pwd))"

# Create the layer to host the extension
LAYER_VER=\
$(aws --profile $AWS_PROFILE lambda publish-layer-version \
    --layer-name  $BASENAME \
    --license-info "MIT" \
    --zip-file "fileb://./target/lambda/extensions/$BASENAME.zip" \
    --compatible-runtimes provided.al2 \
    --compatible-architectures x86_64 arm64 | tr -d \" | grep -Eo "Version: [0-9]+" | grep -Eo "[0-9]+")

# Add the new layer version permissions (currently this makes it available to all AWS accounts)
aws --profile $AWS_PROFILE lambda add-layer-version-permission --layer-name lambda-spy --statement-id public --action lambda:GetLayerVersion  --principal "*" --region $AWS_REGION --version-number $LAYER_VER