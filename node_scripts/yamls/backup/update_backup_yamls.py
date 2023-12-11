import yaml
import os

DOCKER_REGISTRY = os.getenv("DOCKER_REGISTRY")
DOCKER_REPO = os.getenv("DOCKER_REPO")

SHARE_SC = os.getenv("SHARE_SC")

STS_NAME = os.getenv("STS_NAME")
BACKUP_NODE = os.getenv("BACKUP_NODE")

ARGS = os.getenv("ARGS")

# backup_pvc
with open("./yamls/backup/backup_pvc.yaml", "r") as stream:
    try:
        yaml_data = yaml.safe_load(stream)
    except yaml.YAMLError as exc:
        print(exc)

yaml_data["spec"]["storageClassName"] = SHARE_SC

with open("./yamls/backup/backup_pvc.yaml", "w") as outfile:
    try:
        yaml.safe_dump(yaml_data, outfile, default_flow_style=False)
    except yaml.YAMLError as exc:
        print(exc)


# backup_job
with open("./yamls/backup/backup_job.yaml", "r") as stream:
    try:
        yaml_data = yaml.safe_load(stream)
    except yaml.YAMLError as exc:
        print(exc)
cloud_op_command = f'echo "backup start\n" && cloud-op {ARGS} -c ./config/config.toml -n ./source -b ../backup && sleep infinity'
yaml_data["spec"]["template"]["spec"]["containers"][0]["args"][0] = cloud_op_command
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
yaml_data["spec"]["template"]["spec"]["volumes"][2]["configMap"]["name"] = (
    STS_NAME + "-config"
)

with open("./yamls/backup/backup_job.yaml", "w") as outfile:
    try:
        yaml.safe_dump(yaml_data, outfile, default_flow_style=False)
    except yaml.YAMLError as exc:
        print(exc)
