from __future__ import absolute_import
import sys
import json

# shellcheck disable=SC1036  # Hold-over from previous iteration.
def run(previous_task_definition, container_image_name_updates,
        container_env_var_updates, container_secret_updates, container_docker_label_updates):
    try:
        definition = json.loads(previous_task_definition)
        container_definitions = definition['taskDefinition']['containerDefinitions']
    except:
        raise Exception('No valid task definition found: ' + previous_task_definition)

    # Build a map of the original container definitions so that the
    # array index positions can be easily looked up
    container_map = {}
    for index, container_definition in enumerate(container_definitions):
        env_var_map = {}
        env_var_definitions = container_definition.get('environment')
        if env_var_definitions is not None:
            for env_var_index, env_var_definition in enumerate(env_var_definitions):
                env_var_map[env_var_definition['name']] = {'index': env_var_index}
        container_map[container_definition['name']] = {'image': container_definition['image'], 'index': index, 'environment_map': env_var_map}

    # Expected format: container=...,name=...,value=...,container=...,name=...,value=
    try:
        env_kv_pairs = container_env_var_updates.split(',')
        for index, kv_pair in enumerate(env_kv_pairs):
            kv = kv_pair.split('=')
            key = kv[0].strip()

            if key == 'container':
                container_name = kv[1].strip()
                env_var_name_kv = env_kv_pairs[index+1].split('=')
                env_var_name = env_var_name_kv[1].strip()
                env_var_value_kv = env_kv_pairs[index+2].split('=', maxsplit=1)
                env_var_value = env_var_value_kv[1].strip()
                if env_var_name_kv[0].strip() != 'name' or env_var_value_kv[0].strip() != 'value':
                    raise ValueError(
                        'Environment variable update parameter format is incorrect: ' + container_env_var_updates)

                container_entry = container_map.get(container_name)
                if container_entry is None:
                    raise ValueError('The container ' + container_name + ' is not defined in the existing task definition')
                container_index = container_entry['index']
                env_var_entry = container_entry['environment_map'].get(env_var_name)
                if env_var_entry is None:
                    # The existing container definition does not contain environment variables
                    if container_definitions[container_index].get('environment') is None:
                        container_definitions[container_index]['environment'] = []
                    # This env var does not exist in the existing container definition
                    container_definitions[container_index]['environment'].append({'name': env_var_name, 'value': env_var_value})
                else:
                    env_var_index = env_var_entry['index']
                    container_definitions[container_index]['environment'][env_var_index]['value'] = env_var_value
            elif key and key not in ['container', 'name', 'value']:
                raise ValueError('Incorrect key found in environment variable update parameter: ' + key)
    except ValueError as value_error:
        raise value_error
    except:
        raise Exception('Environment variable update parameter could not be processed; please check parameter value: ' + container_env_var_updates)

    # Expected format: container=...,string=...,string=...,container=...,string=...,string=
    
    try:
        docker_label_kv_pairs = container_docker_label_updates.split(',')
        for index, kv_pair in enumerate(docker_label_kv_pairs):
            kv = kv_pair.split('=')
            key = kv[0].strip()

            if key == 'container':
                container_name = kv[1].strip()
                docker_label_kv = docker_label_kv_pairs[index+1].split('=')
                docker_label_key = docker_label_kv[0].strip()
                docker_label_value = docker_label_kv[1].strip()
                container_entry = container_map.get(container_name)
                if container_entry is None:
                    raise ValueError('The container ' + container_name + ' is not defined in the existing task definition')
                container_index = container_entry['index']
                docker_label_entry = container_entry['environment_map'].get(docker_label_key)
                if docker_label_entry is None:
                    # The existing container definition does not contain environment variables
                    if container_definitions[container_index].get('dockerLabels') is None:
                        container_definitions[container_index]['dockerLabels'] = {}
                    # This env var does not exist in the existing container definition
                    container_definitions[container_index]['dockerLabels'][docker_label_key] =  docker_label_value
                else:
                    docker_label_index = docker_label_entry['index']
                    container_definitions[container_index]['dockerLabels'][docker_label_index][docker_label_key] = docker_label_value
    except ValueError as value_error:
        raise value_error
    except:
        raise Exception('Docker label update parameter could not be processed; please check parameter value: ' + container_docker_label_updates)

    # Expected format: container=...,name=...,valueFrom=...,container=...,name=...,valueFrom=...

    try:
        secret_kv_pairs = container_secret_updates.split(',')
        for index, kv_pair in enumerate(secret_kv_pairs):
            kv = kv_pair.split('=')
            key = kv[0].strip()
            if key == 'container':
                container_name = kv[1].strip()
                secret_name_kv = secret_kv_pairs[index+1].split('=')
                secret_name = secret_name_kv[1].strip()
                secret_value_kv = secret_kv_pairs[index+2].split('=', maxsplit=1)
                secret_value = secret_value_kv[1].strip()
                if secret_name_kv[0].strip() != 'name' or secret_value_kv[0].strip() != 'valueFrom':
                    raise ValueError(
                        'Container secret update parameter format is incorrect: ' + container_secret_updates)

                container_entry = container_map.get(container_name)
                if container_entry is None:
                    raise ValueError('The container ' + container_name + ' is not defined in the existing task definition')
                container_index = container_entry['index']
                secret_entry = container_entry['environment_map'].get(secret_name)
                if secret_entry is None:
                    # The existing container definition does not contain secrets variable
                    if container_definitions[container_index].get('secrets') is None:
                        container_definitions[container_index]['secrets'] = []
                    # The secrets variable does not exist in the existing container definition
                    container_definitions[container_index]['secrets'].append({'name': secret_name, 'valueFrom': secret_value})
                else:
                    secret_index = secret_entry['index']
                    container_definitions[container_index]['secrets'][secret_index]['valueFrom'] = secret_value
            elif key and key not in ['container', 'name', 'valueFrom']:
                raise ValueError('Incorrect key found in secret updates parameter: ' + key)
    except ValueError as value_error:
        raise value_error
    except:
        raise Exception('Container secrets update parameter could not be processed; please check parameter value: ' + container_secret_updates)

    # Expected format: container=...,image-and-tag|image|tag=...,container=...,image-and-tag|image|tag=...,
    try:
        if container_image_name_updates and "container=" not in container_image_name_updates:
            raise ValueError('The container parameter is required in the container_image_name_updates variable.')

        image_kv_pairs = container_image_name_updates.split(',')
        for index, kv_pair in enumerate(image_kv_pairs):
            kv = kv_pair.split('=')
            key = kv[0].strip()
            if key == 'container':
                container_name = kv[1].strip()
                image_kv = image_kv_pairs[index+1].split('=')
                container_entry = container_map.get(container_name)
                if container_entry is None:
                    raise ValueError('The container ' + container_name + ' is not defined in the existing task definition')
                container_index = container_entry['index']
                image_specifier_type = image_kv[0].strip()
                image_value = image_kv[1].strip()
                if image_specifier_type == 'image-and-tag':
                    container_definitions[container_index]['image'] = image_value
                else:
                    existing_image_name_tokens = container_entry['image'].split(':')
                    if image_specifier_type == 'image':
                        tag = ''
                        if len(existing_image_name_tokens) == 2:
                            tag = ':' + existing_image_name_tokens[1]
                        container_definitions[container_index]['image'] = image_value + tag
                    elif image_specifier_type == 'tag':
                        container_definitions[container_index]['image'] = existing_image_name_tokens[0] + ':' + image_value
                    else:
                        raise ValueError(
                            'Image name update parameter format is incorrect: ' + container_image_name_updates)
            elif key and key not in ['container', 'image', 'image-and-tag', 'tag']:
                raise ValueError('Incorrect key found in image name update parameter: ' + key)

    except ValueError as value_error:
        raise value_error
    except:
        raise Exception('Image name update parameter could not be processed; please check parameter value: ' + container_image_name_updates)
    return json.dumps(container_definitions)


if __name__ == '__main__':
    try:
        print(run(sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5]))
    except Exception as e:
        sys.stderr.write(str(e) + "\n")
        exit(1)
