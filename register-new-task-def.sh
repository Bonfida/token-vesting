set -o noglob

# These variables are evaluated so the config file may contain and pass in environment variables to the parameters.
ECS_PARAM_FAMILY=$(eval echo "$ECS_PARAM_FAMILY")
ECS_PARAM_PROFILE_NAME=$(eval echo "$ECS_PARAM_PROFILE_NAME")

if [ -n "${CCI_ORB_AWS_ECS_TASK_ROLE}" ]; then
    set -- "$@" --task-role-arn "${CCI_ORB_AWS_ECS_TASK_ROLE}"
fi

if [ -n "${CCI_ORB_AWS_ECS_EXECUTION_ROLE}" ]; then
    set -- "$@" --execution-role-arn "${CCI_ORB_AWS_ECS_EXECUTION_ROLE}"
fi

if [ -n "${CCI_ORB_AWS_ECS_NETWORK_MODE}" ]; then
    set -- "$@" --network-mode "${CCI_ORB_AWS_ECS_NETWORK_MODE}"
fi

if [ -n "${CCI_ORB_AWS_ECS_VOLUMES}" ] && [ "${CCI_ORB_AWS_ECS_VOLUMES}" != "[]" ]; then
    set -- "$@" --volumes "${CCI_ORB_AWS_ECS_VOLUMES}"
fi

if [ -n "${CCI_ORB_AWS_ECS_PLACEMENT_CONSTRAINTS}" ] && [ "${CCI_ORB_AWS_ECS_PLACEMENT_CONSTRAINTS}" != "[]" ]; then
    set -- "$@" --placement-constraints "${CCI_ORB_AWS_ECS_PLACEMENT_CONSTRAINTS}"
fi

if [ -n "${CCI_ORB_AWS_ECS_REQ_COMP}" ] && [ "${CCI_ORB_AWS_ECS_REQ_COMP}" != "[]" ]; then
    #shellcheck disable=SC2086
    set -- "$@" --requires-compatibilities ${CCI_ORB_AWS_ECS_REQ_COMP}
fi

if [ -n "${CCI_ORB_AWS_ECS_TASK_CPU}" ]; then
    set -- "$@" --cpu "${CCI_ORB_AWS_ECS_TASK_CPU}"
fi

if [ -n "${CCI_ORB_AWS_ECS_TASK_MEMORY}" ]; then
    set -- "$@" --memory "${CCI_ORB_AWS_ECS_TASK_MEMORY}"
fi

if [ -n "${CCI_ORB_AWS_ECS_PID_MODE}" ]; then
    set -- "$@" --pid-mode "${CCI_ORB_AWS_ECS_PID_MODE}"
fi

if [ -n "${CCI_ORB_AWS_ECS_IPC_MODE}" ]; then
    set -- "$@" --ipc-mode "${CCI_ORB_AWS_ECS_IPC_MODE}"
fi

if [ -n "${CCI_ORB_AWS_ECS_TAGS}" ] && [ "${CCI_ORB_AWS_ECS_TAGS}" != "[]" ]; then
    set -- "$@" --tags "${CCI_ORB_AWS_ECS_TAGS}"
fi

if [ -n "${CCI_ORB_AWS_ECS_PROXY_CONFIGURATION}" ] && [ "${CCI_ORB_AWS_ECS_PROXY_CONFIGURATION}" != "{}" ]; then
    set -- "$@" --proxy-configuration "${CCI_ORB_AWS_ECS_PROXY_CONFIGURATION}"
fi

if [ -n "${ECS_PARAM_PROFILE_NAME}" ]; then
    set -- "$@" --profile "${ECS_PARAM_PROFILE_NAME}"
fi

if [ -n "${CCI_ORB_AWS_ECS_RUNTIME_PLATFORM}" ] && [ "${CCI_ORB_AWS_ECS_RUNTIME_PLATFORM}" != "{}" ]; then
    set -- "$@" --runtime-platform "${CCI_ORB_AWS_ECS_RUNTIME_PLATFORM}"
fi

if [ -n "${CCI_ORB_AWS_ECS_EPHEMERAL_STORAGE}" ] && [ "${CCI_ORB_AWS_ECS_EPHEMERAL_STORAGE}" != "{}" ]; then
    set -- "$@" --ephemeral-storage "${CCI_ORB_AWS_ECS_EPHEMERAL_STORAGE}"
fi

REVISION=$(aws ecs register-task-definition \
    --family "$ECS_PARAM_FAMILY" \
    --container-definitions "${CCI_ORB_AWS_ECS_CONTAINER_DEFS}" \
    "$@" \
    --output text \
    --query 'taskDefinition.taskDefinitionArn')
echo "Registered task definition: ${REVISION}"

echo "export CCI_ORB_AWS_ECS_REGISTERED_TASK_DFN='${REVISION}'" >> "$BASH_ENV"
