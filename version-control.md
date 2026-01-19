## Git
- collab
- version control (maintain history)
- distributed vcs

### Initialize Repo
- git init
- git remote -v
- git remote add origin url

### Commands
- git add .
- git add file_name
- git commit -m "message"
- git commit -am "message"
- git push -u origin master
- git push origin branch_name
- git pull origin branch_name
- git merge branch_name

- git status
- git log --oneline
- git diff
- git reset #hash_id
- git reset --hard #hash_id

### Branch and Merge
- git branch branch_name
- git checkout branch_name
- git checkout -b branch_name


### Collab
- fork repo & clone fork repo
- git clone https://github.com/your-username/their-repo.git
- git remote add upstream https://github.com/their-username/their-repo.git
- git fetch upstream
- git checkout -b feature/short-descriptive-name
- git add .
- git commit -m "Short summary of change"
- git push origin feature/short-descriptive-name
- add PR


### Comments:
- feat: added new feature
- fix: fixed bug & issue
- style: changed formattting and UI
- refactor: improved code structure
- docs: chnages releated to documentation
- chore: minor code keeping (update and rename folder)