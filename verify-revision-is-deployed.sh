# These variables are evaluated so the config file may contain and pass in environment variables to the parameters.
ECS_PARAM_FAMILY=$(eval echo "$ECS_PARAM_FAMILY")
ECS_PARAM_SERVICE_NAME=$(eval echo "$ECS_PARAM_SERVICE_NAME")
ECS_PARAM_CLUSTER_NAME=$(eval echo "$ECS_PARAM_CLUSTER_NAME")
ECS_PARAM_TASK_DEF_ARN=$(eval echo "$ECS_PARAM_TASK_DEF_ARN")
ECS_PARAM_PROFILE_NAME=$(eval echo "$ECS_PARAM_PROFILE_NAME")

if [ "$ECS_PARAM_TASK_DEF_ARN" = "" ]; then
    echo "Invalid task-definition-arn parameter value: $ECS_PARAM_TASK_DEF_ARN"
    exit 1
fi


if [ -z "${ECS_PARAM_SERVICE_NAME}" ]; then
    ECS_PARAM_SERVICE_NAME="$ECS_PARAM_FAMILY"
fi

echo "Verifying that $ECS_PARAM_TASK_DEF_ARN is deployed.."

attempt=0

while [ "$attempt" -lt "$ECS_PARAM_MAX_POLL_ATTEMPTS" ]

do  
    if [ -n "${ECS_PARAM_PROFILE_NAME}" ]; then
        set -- "$@" --profile="${ECS_PARAM_PROFILE_NAME}"   
    fi

    DEPLOYMENTS=$(aws ecs describe-services \
        --cluster "$ECS_PARAM_CLUSTER_NAME" \
        --services "${ECS_PARAM_SERVICE_NAME}" \
        --output text \
        --query 'services[0].deployments[].[taskDefinition, status]' \
        "$@")
    NUM_DEPLOYMENTS=$(aws ecs describe-services \
        --cluster "$ECS_PARAM_CLUSTER_NAME" \
        --services "${ECS_PARAM_SERVICE_NAME}" \
        --output text \
        --query 'length(services[0].deployments)' \
        "$@")
    TARGET_REVISION=$(aws ecs describe-services \
        --cluster "$ECS_PARAM_CLUSTER_NAME" \
        --services "${ECS_PARAM_SERVICE_NAME}" \
        --output text \
        --query "services[0].deployments[?taskDefinition==\`$ECS_PARAM_TASK_DEF_ARN\` && runningCount == desiredCount && (status == \`PRIMARY\` || status == \`ACTIVE\`)][taskDefinition]" \
        "$@")
    echo "Current deployments: $DEPLOYMENTS"
    if [ "$NUM_DEPLOYMENTS" = "1" ] && [ "$TARGET_REVISION" = "$ECS_PARAM_TASK_DEF_ARN" ]; then
        echo "The task definition revision $TARGET_REVISION is the only deployment for the service and has attained the desired running task count."
        exit 0
    else
        echo "Waiting for revision $ECS_PARAM_TASK_DEF_ARN to reach desired running count / older revisions to be stopped.."
        sleep "$ECS_PARAM_POLL_INTERVAL"
    fi
    attempt=$((attempt + 1))
done

echo "Stopped waiting for deployment to be stable - please check the status of $ECS_PARAM_TASK_DEF_ARN on the AWS ECS console."

if [ "$ECS_PARAM_FAIL_ON_VERIFY_TIMEOUT" = "1" ]; then
    exit 1
fi

