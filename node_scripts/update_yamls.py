import yaml
import os

DOCKER_REGISTRY = os.getenv("DOCKER_REGISTRY")
DOCKER_REPO = os.getenv("DOCKER_REPO")

NEW_NODE_SC = os.getenv("NEW_NODE_SC")
SHARE_SC = os.getenv("SHARE_SC")

BACKUP_NODE = os.getenv("BACKUP_NODE")
NEW_NODE = os.getenv("NEW_NODE")
STS_NAME = os.getenv("STS_NAME")

# backup_pvc
with open("./yamls/backup_pvc.yaml", "r") as stream:
    try:
        yaml_data = yaml.safe_load(stream)
    except yaml.YAMLError as exc:
        print(exc)

yaml_data["spec"]["storageClassName"] = SHARE_SC

with open("./yamls/backup_pvc.yaml", "w") as outfile:
    try:
        yaml.safe_dump(yaml_data, outfile, default_flow_style=False)
    except yaml.YAMLError as exc:
        print(exc)


# backup_job
with open("./yamls/backup_job.yaml", "r") as stream:
    try:
        yaml_data = yaml.safe_load(stream)
    except yaml.YAMLError as exc:
        print(exc)

yaml_data["spec"]["template"]["spec"]["containers"][0]["image"] = (
    DOCKER_REGISTRY + "/" + DOCKER_REPO + "/cloud-op:latest"
)
yaml_data["spec"]["template"]["spec"]["containers"][0]["volumeMounts"][1]["name"] = (
    "datadir-" + BACKUP_NODE
)
yaml_data["spec"]["template"]["spec"]["volumes"][1]["name"] = "datadir-" + BACKUP_NODE
yaml_data["spec"]["template"]["spec"]["volumes"][1]["persistentVolumeClaim"][
    "claimName"
] = ("datadir-" + BACKUP_NODE)
yaml_data["spec"]["template"]["spec"]["volumes"][2]["configMap"][
    "name"
] = (STS_NAME + "-config")

with open("./yamls/backup_job.yaml", "w") as outfile:
    try:
        yaml.safe_dump(yaml_data, outfile, default_flow_style=False)
    except yaml.YAMLError as exc:
        print(exc)


# node_pvc
with open("./yamls/node_pvc.yaml", "r") as stream:
    try:
        yaml_data = yaml.safe_load(stream)
    except yaml.YAMLError as exc:
        print(exc)

yaml_data["metadata"]["name"] = "datadir-" + NEW_NODE
yaml_data["spec"]["storageClassName"] = NEW_NODE_SC

with open("./yamls/node_pvc.yaml", "w") as outfile:
    try:
        yaml.safe_dump(yaml_data, outfile, default_flow_style=False)
    except yaml.YAMLError as exc:
        print(exc)


# recover_job
with open("./yamls/recover_job.yaml", "r") as stream:
    try:
        yaml_data = yaml.safe_load(stream)
    except yaml.YAMLError as exc:
        print(exc)

yaml_data["spec"]["template"]["spec"]["containers"][0]["volumeMounts"][0]["name"] = (
    "datadir-" + NEW_NODE
)
yaml_data["spec"]["template"]["spec"]["volumes"][1]["name"] = "datadir-" + NEW_NODE
yaml_data["spec"]["template"]["spec"]["volumes"][1]["persistentVolumeClaim"][
    "claimName"
] = ("datadir-" + NEW_NODE)

with open("./yamls/recover_job.yaml", "w") as outfile:
    try:
        yaml.safe_dump(yaml_data, outfile, default_flow_style=False)
    except yaml.YAMLError as exc:
        print(exc)
