use super::{
    ClientError, Command, Easing, Kind, MAX_DURATION, Mc, Panel, PropsUpdate, Put, RpcError, Rule,
    Say, SetHud, SetPanel, TICK, Typing, agent, in_place, invalid, page_id, panel_id, push_word,
    region, scene,
};

impl Mc {
    /// Keeps the lines of the text contents that stay, and takes those of
    /// the settings for new ones.
    pub(crate) fn keep_texts(&mut self) {
        let contents = &self.settings.value.contents;
        let old = std::mem::take(&mut self.texts);
        self.texts = (contents.iter())
            .filter(|(_, c)| c.kind == Kind::Text)
            .map(|(name, c)| {
                let text = old
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| (c.title.clone(), c.lines.clone()));
                (name.clone(), text)
            })
            .collect();
    }

    /// Changes the lines of a text content, and its title, for
    /// `content.set` and `content.add`.
    pub(crate) fn set_text(
        &mut self,
        content: &str,
        title: Option<String>,
        change: impl FnOnce(Vec<String>) -> Vec<String>,
    ) -> Result<(), RpcError> {
        let (old_title, lines) = (self.texts.get(content).cloned())
            .ok_or_else(|| invalid(format!("no text content {content}")))?;
        let new = (title.unwrap_or(old_title), change(lines));
        if new.0.len() + new.1.join("\n").len() > scene::MAX_TEXT {
            return Err(invalid(format!("{content}: more than 4 KiB of text")));
        }
        self.texts.insert(content.into(), new);
        self.render()
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))
    }

    /// Puts a content into a region over the rules, or back to them.
    pub(crate) fn put_content(&mut self, put: Put) -> Result<(), RpcError> {
        let Put {
            content,
            region,
            slot,
        } = put;
        let found = (self.settings.value.contents.get(&content))
            .ok_or_else(|| invalid(format!("no content {content}")))?;
        if let Some(region) = region {
            region::put_check(found, region).map_err(invalid)?;
        }
        self.put.retain(|r| r.content.as_ref() != Some(&content));
        if let Some(region) = region {
            let rule = Rule {
                content: Some(content),
                kind: None,
                scenes: Vec::new(),
                region,
                slot,
            };
            self.put.insert(0, rule);
        }
        Ok(())
    }

    pub(crate) fn set_panel(&mut self, set: SetPanel) -> Result<(), RpcError> {
        let SetPanel {
            scene,
            panel,
            title,
            lines,
        } = set;
        if agent::OWN_SCENES.contains(&scene.as_str()) {
            return Err(invalid(format!(
                "the panels of {scene} are written by mc alone"
            )));
        }
        let found = (self.scenes.get(&scene))
            .and_then(|v| v.panels.iter().find(|p| p.id == panel))
            .ok_or_else(|| invalid(format!("no panel {panel} in scene {scene}")))?;
        let new = Panel {
            title: title.unwrap_or_else(|| found.title.clone()),
            lines,
            ..found.clone()
        };
        self.replace_panel(&scene, &new, true)
    }

    /// Puts a changed panel into its scene, by its id; it writes itself
    /// when the scene shows.
    /// Without `write` the panel changes in place when it keeps its nodes,
    /// so that an agent state does not blank and write its column anew.
    /// The same text again neither writes nor builds anything.
    /// A panel that moves, fills or empties moves the panels after it
    /// on a narrow page: the page builds anew.
    pub fn replace_panel(&mut self, scene: &str, new: &Panel, write: bool) -> Result<(), RpcError> {
        new.check_text().map_err(invalid)?;
        let found = (self.scenes.get_mut(scene))
            .and_then(|v| v.panels.iter_mut().find(|p| p.id == new.id))
            .ok_or_else(|| invalid(format!("no panel {} in scene {scene}", new.id)))?;
        if found == new {
            return Ok(());
        }
        let moved = found.cell != new.cell || found.is_empty() != new.is_empty();
        let old = std::mem::replace(found, new.clone());
        let scene = scene.to_owned();
        let shown = self.current.as_deref() == Some(&scene);
        let commands = if moved {
            let mut commands = vec![Command::Remove {
                id: page_id(&scene),
                to: None,
            }];
            commands.extend(self.panels(&scene));
            if write && shown {
                commands.extend(self.write(&scene, &[new]));
            }
            commands
        } else {
            let created = self.panel(&scene, new);
            let updates = (!write)
                .then(|| in_place(&self.panel(&scene, &old), &created))
                .flatten();
            let mut commands = updates.unwrap_or_else(|| {
                let mut commands = vec![Command::Remove {
                    id: panel_id(&scene, &new.id),
                    to: None,
                }];
                commands.extend(created);
                if write && shown {
                    commands.extend(self.write(&scene, &[new]));
                }
                commands
            });
            commands.push(Command::Remove {
                id: format!("{}-rule", panel_id(&scene, &new.id)),
                to: None,
            });
            commands.extend(self.rule(&scene, new));
            commands
        };
        self.apply(commands)
            .and_then(|()| self.align(&scene))
            .and_then(|()| {
                if moved && shown {
                    self.take_free_keyboard()
                } else {
                    Ok(())
                }
            })
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))
    }

    /// Shows the rest of `hud.say` at once: the next build has a new tick
    /// node without an animation.
    pub(crate) fn finish_typing(&mut self) {
        if let Some(typing) = self.typing.take() {
            for (word, _) in &typing.words[typing.shown..] {
                push_word(&mut self.hud.text, word);
            }
        }
    }

    /// Starts `hud.say`: the first word shows now, each next one after
    /// the time of the word before, by its share of the letters.
    /// at most 4 KiB of text
    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn say(&mut self, say: Say) -> Result<(), ClientError> {
        let words: Vec<&str> = say.text.split_whitespace().collect();
        let letters: usize = words.iter().map(|w| w.chars().count() + 1).sum();
        let ms = say.duration_ms.clamp(0.0, MAX_DURATION) / letters.max(1) as f64;
        let words = words
            .iter()
            .map(|w| ((*w).to_owned(), ms * (w.chars().count() + 1) as f64))
            .collect();
        self.hud = SetHud {
            state: say.state,
            text: String::new(),
        };
        self.typing = Some(Typing { words, shown: 0 });
        self.type_next()
    }

    /// Shows the next word of `hud.say` and waits for the one after it.
    pub(crate) fn type_next(&mut self) -> Result<(), ClientError> {
        let Some(typing) = &mut self.typing else {
            return Ok(());
        };
        let Some((word, ms)) = typing.words.get(typing.shown).cloned() else {
            self.typing = None;
            return Ok(());
        };
        typing.shown += 1;
        push_word(&mut self.hud.text, &word);
        let tick = Command::Animate {
            id: TICK.into(),
            props: PropsUpdate::default(),
            duration: ms,
            easing: Easing::Linear,
        };
        self.apply(vec![tick])?;
        self.render()
    }
}
