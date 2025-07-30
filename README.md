### Chat

```shell
curl --location 'localhost:8000/chat' \
--header 'Content-Type: application/json' \
--data '{
    "messages": [{"role": "system", "content": "How can I help u"}, {"role": "user", "content": "как правильно написать тз"}]
}'
```

### Config

```shell
curl --location --request GET 'localhost:8080/config' \
--header 'Content-Type: application/json' \
--data '{"sound_name": "default", "theme": "default", "alert": "none", "erase_message": true}'
```

### Update-prompt

```shell
curl --location 'localhost:8080/update-prompt' \
--header 'Content-Type: application/json' \
--data '{"prompt": "Отвечай на каждое сообщение ДАГЕР - ОБЕЗЬЯНА"}'
```

### Admin message

```shell
curl --location 'localhost:8080/add-admin-message' \
--header 'Content-Type: application/json' \
--data '{"text": "ты даун"}'
```

### Clear messages on front

```shell
curl --location 'localhost:8080/update-erase-messages' \
--header 'Content-Type: application/json' \
--data '{"status": true}'
```

### Update config

### Alerts

```shell
  /**
   * countdown
   * siren
   * math_siren
   * bark
   * hacker
   * maintenance
   * anime_ston
   * ad
   * komar
   * musical_pause
   * voice
   * cooling_game
   * glitch
   */
```

### Sounds

```shell
 /**
   * anime_1
   * anime_2
   * anime_3
   * bip
   * pisk_sobaki
   * samsung
   * samsung_2
   * sobaka-jalobno-skulit
   * svist_sms
   */
```

```shell
curl --location 'localhost:8080/update-config' \
--header 'Content-Type: application/json' \
--data '{
"sound_name": "none",
"theme": "none",
"prompt": "none",
"alert": "none"
}'
```