# Tableflow-rs

## Preparation
1. First and most: Install Rust environment:
   1. Go to official page: https://www.rust-lang.org/ to download installation pkg
2. CLion + Rust Plugin;
3. Tilt env

## Start up
### Configuration
#### Coordinator
In etc/coord.json file, you can see the cluster configuration like:
```json
{
   "cluster": [
      {
         "host": "SHA-MAC-JASON-SONG-0.local",
         "port": 18030
      }
   ]
}
```

The host must be your computer's hostname. For macOS, you can execute command in terminal: ``hostname``

#### Worker

## For Contribution
1. For now, we use single-branch development model. Every dev should develop on master branch. 
2. For release branch, only admin has authority to initiate pull request. Then he/she will wait for all relevant dev to affirm;
3. Deployment will be triggered automatically after a release tag is published;

### How to release New Tag
**Tag's name template**:

* For test (master branch): ``dev.v{yyyyMMdd}.{version}``
* For production (release branch): ``prod.v{yyyyMMdd}.{version}``
