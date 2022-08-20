#!/bin/bash

main=README.md
temp=/tmp/README.md

BEGIN_GEN=$(cat $main | grep -n '<!-- BEGIN INSTALLATION -->' | sed 's/\(.*\):.*/\1/g')
END_GEN=$(cat $main | grep -n '<!-- END INSTALLATION -->' | sed 's/\(.*\):.*/\1/g')

cat <(head -n $(expr $BEGIN_GEN) $main)                         > $temp
echo '```bash'                                                  >> $temp
echo '$ cargo install --git https://github.com/duyet/athena-rs' >> $temp
echo '$ athena --help'                                          >> $temp
echo ''                                                         >> $temp
cargo run -q -- help                                            >> $temp
echo '```'                                                      >> $temp
cat <(tail -n +$(expr $END_GEN) $main)                          >> $temp

cat $temp
cat $temp > $main

# Commit changes
CURRENT_BRANCH=${GITHUB_REF#refs/heads/}
REMOTE_REPO="https://${GITHUB_ACTOR}:${{ secrets.GITHUB_TOKEN }}@github.com/${GITHUB_REPOSITORY}.git"
git config --local user.email "duyetbot@users.noreply.github.com"
git config --local user.name "duyetbot"
git status
git add README.md
git commit -m "docs: Generate README.md [skip ci]"
git push $REMOTE_REPO $CURRENT_BRANCH
