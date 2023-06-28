# git branch cli
git 브랜치 관련 쉘 스크립트 코드를 대체 하기 위해 만듬

## require
- git 설치

## sub command
### delete
- 로컬 브랜치에서 삭제할 브랜치를 선택해 삭제 할 수 있도록함
```
some_project> ../git_branch delete
? Select branch list to delete:  
> [ ] [2023-01-01 00:00:00] main
[↑↓ to move, space to select one, → to all, ← to none, type to filter]
```
- 삭재한 브랜치를 선택 하고 space bar 를 누르면 선택됨
- 다 선택하고 enter 를 누르면 선택된 브랜치가 삭제됨

### recommand
- `<STAGE>/YYYYMMDD.1` 같은 형태로 브랜치 명을 만들어줌
- remote 에 오늘자 브랜치가 있는지 확인 하고 없으면 `stage/20230101.1` 로 있으면, 기존 버전 +1 해줌 (예: `stage/20230101.2`)