td_arn=$CCI_ORB_AWS_ECS_REGISTERED_TASK_DFN

if [ -z "$td_arn" ]; then
    echo "Updated task definition not found. Please run update-task-definition command before deploy-ecs-scheduled-task"
    exit 1
fi

CLI_OUTPUT_FILE=$(mktemp cli-output.json.XXXX)
CLI_INPUT_FILE=$(mktemp cli-input.json.XXXX)

aws events list-targets-by-rule --rule "$ECS_PARAM_RULE_NAME" --output json > "$CLI_OUTPUT_FILE"

if < "$CLI_OUTPUT_FILE" jq ' .Targets[] | has("EcsParameters")' | grep "false"; then
    echo "Invalid ECS Rule. $ECS_PARAM_RULE_NAME does not contain EcsParameters key. Please create a valid ECS Rule and try again"
    exit 1
fi

< "$CLI_OUTPUT_FILE" jq --arg td_arn "$td_arn" '.Targets[].EcsParameters.TaskDefinitionArn |= $td_arn' > "$CLI_INPUT_FILE"
aws events put-targets --rule $ECS_PARAM_RULE_NAME --cli-input-json "$(cat "$CLI_INPUT_FILE")"
