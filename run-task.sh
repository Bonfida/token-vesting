if [[ $EUID == 0 ]]; then export SUDO=""; else export SUDO="sudo"; fi
# These variables are evaluated so the config file may contain and pass in environment variables to the parameters.
ECS_PARAM_CLUSTER_NAME=$(eval echo "$ECS_PARAM_CLUSTER_NAME")
ECS_PARAM_TASK_DEF=$(eval echo "$ECS_PARAM_TASK_DEF")
ECS_PARAM_PROFILE_NAME=$(eval echo "$ECS_PARAM_PROFILE_NAME")

if ! command -v envsubst && [[ "$ECS_PARAM_OVERRIDES" == *"\${"* ]]; then
    echo "Installing envsubst."
    curl -L https://github.com/a8m/envsubst/releases/download/v1.2.0/envsubst-"$(uname -s)"-"$(uname -m)" -o envsubst
    $SUDO chmod +x envsubst
    $SUDO mv envsubst /usr/local/bin
    ECS_PARAM_OVERRIDES=$(echo "${ECS_PARAM_OVERRIDES}" | envsubst)
fi

set -o noglob
if [ -n "$ECS_PARAM_PLATFORM_VERSION" ]; then
    echo "Setting --platform-version"
    set -- "$@" --platform-version "$ECS_PARAM_PLATFORM_VERSION"
fi
if [ -n "$ECS_PARAM_STARTED_BY" ]; then
    echo "Setting --started-by"
    set -- "$@" --started-by "$ECS_PARAM_STARTED_BY"
fi
if [ -n "$ECS_PARAM_GROUP" ]; then
    echo "Setting --group"
    set -- "$@" --group "$ECS_PARAM_GROUP"
fi
if [ -n "$ECS_PARAM_OVERRIDES" ]; then
    echo "Setting --overrides"
    set -- "$@" --overrides "$ECS_PARAM_OVERRIDES"
fi
if [ -n "$ECS_PARAM_TAGS" ]; then
    echo "Setting --tags"
    set -- "$@" --tags "$ECS_PARAM_TAGS"
fi
if [ -n "$ECS_PARAM_PLACEMENT_CONSTRAINTS" ]; then
    echo "Setting --placement-constraints"
    set -- "$@" --placement-constraints "$ECS_PARAM_PLACEMENT_CONSTRAINTS"
fi
if [ -n "$ECS_PARAM_PLACEMENT_STRATEGY" ]; then
    echo "Setting --placement-strategy"
    set -- "$@" --placement-strategy "$ECS_PARAM_PLACEMENT_STRATEGY"
fi
if [ "$ECS_PARAM_ENABLE_ECS_MANAGED_TAGS" == "1" ]; then
    echo "Setting --enable-ecs-managed-tags"
    set -- "$@" --enable-ecs-managed-tags
fi
if [ "$ECS_PARAM_PROPAGATE_TAGS" == "1" ]; then
    echo "Setting --propagate-tags"
    set -- "$@" --propagate-tags "TASK_DEFINITION"
fi
if [ "$ECS_PARAM_AWSVPC" == "1" ]; then
    echo "Setting --network-configuration"
    if [ -z "$ECS_PARAM_SUBNET_ID" ]; then
        echo '"subnet-ids" is missing.'
        echo 'When "awsvpc" is enabled, "subnet-ids" must be provided.'
        exit 1
    fi
    ECS_PARAM_SUBNET_ID=$(eval echo "$ECS_PARAM_SUBNET_ID")
    ECS_PARAM_SEC_GROUP_ID=$(eval echo "$ECS_PARAM_SEC_GROUP_ID")
    set -- "$@" --network-configuration awsvpcConfiguration="{subnets=[$ECS_PARAM_SUBNET_ID],securityGroups=[$ECS_PARAM_SEC_GROUP_ID],assignPublicIp=$ECS_PARAM_ASSIGN_PUB_IP}"
fi
if [ -n "$ECS_PARAM_CD_CAPACITY_PROVIDER_STRATEGY" ]; then
    echo "Setting --capacity-provider-strategy"
    # do not quote
    # shellcheck disable=SC2086
    set -- "$@" --capacity-provider-strategy $ECS_PARAM_CD_CAPACITY_PROVIDER_STRATEGY
fi

if [ -n "$ECS_PARAM_LAUNCH_TYPE" ]; then
    if [ -n "$ECS_PARAM_CD_CAPACITY_PROVIDER_STRATEGY" ]; then
        echo "Error: "
        echo 'If a "capacity-provider-strategy" is specified, the "launch-type" parameter must be set to an empty string.'
        exit 1
    else
        echo "Setting --launch-type"
        set -- "$@" --launch-type "$ECS_PARAM_LAUNCH_TYPE"
    fi
fi

if [ -n "${ECS_PARAM_PROFILE_NAME}" ]; then
    set -- "$@" --profile "${ECS_PARAM_PROFILE_NAME}"
fi

echo "Setting --count"
set -- "$@" --count "$ECS_PARAM_COUNT"
echo "Setting --task-definition"
set -- "$@" --task-definition "$ECS_PARAM_TASK_DEF"
echo "Setting --cluster"
set -- "$@" --cluster "$ECS_PARAM_CLUSTER_NAME"


if [ -n "${ECS_PARAM_RUN_TASK_OUTPUT}" ]; then
    aws ecs run-task "$@" | tee "${ECS_PARAM_RUN_TASK_OUTPUT}"
else    
    aws ecs run-task "$@"
fi
