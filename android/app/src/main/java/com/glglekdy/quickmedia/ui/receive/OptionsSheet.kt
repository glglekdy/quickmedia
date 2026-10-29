package com.glglekdy.quickmedia.ui.receive

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.glglekdy.quickmedia.options.DownloadOptions
import com.glglekdy.quickmedia.options.PlaylistMode
import com.glglekdy.quickmedia.ui.components.SegmentToggle
import com.glglekdy.quickmedia.ui.theme.Qm

/** 자주 안 쓰는 옵션: 재생목록, 썸네일·메타데이터, 구간 자르기 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun OptionsSheet(
    o: DownloadOptions,
    errors: Map<String, String>,
    onChange: (DownloadOptions) -> Unit,
    onDismiss: () -> Unit,
) {
    val c = Qm.colors
    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
        containerColor = c.surface,
    ) {
        Column(
            Modifier.padding(horizontal = 20.dp).padding(bottom = 24.dp).navigationBarsPadding(),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Text("옵션", color = c.text, fontSize = 20.sp, fontWeight = FontWeight.ExtraBold)

            Label("재생목록")
            SegmentToggle(
                values = PlaylistMode.entries,
                selected = o.playlist,
                label = {
                    when (it) {
                        PlaylistMode.SINGLE -> "이 영상만"
                        PlaylistMode.ALL -> "전체"
                        PlaylistMode.ITEMS -> "일부 항목"
                    }
                },
                onSelect = { onChange(o.copy(playlist = it)) },
            )
            if (o.playlist == PlaylistMode.ITEMS) {
                OptionField(o.playlistItems, "받을 항목", "1-5,8", errors["playlistItems"], Modifier.fillMaxWidth()) {
                    onChange(o.copy(playlistItems = it))
                }
            }

            Row(
                Modifier.fillMaxWidth().clickable { onChange(o.copy(embedMeta = !o.embedMeta)) }.padding(vertical = 6.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text("썸네일·메타데이터 넣기", color = c.text, fontSize = 15.sp, modifier = Modifier.weight(1f))
                Switch(
                    checked = o.embedMeta,
                    onCheckedChange = { onChange(o.copy(embedMeta = it)) },
                    colors = SwitchDefaults.colors(
                        checkedThumbColor = Color.White,
                        checkedTrackColor = c.accent,
                        uncheckedThumbColor = c.textDim,
                        uncheckedTrackColor = c.field,
                        uncheckedBorderColor = c.field,
                    ),
                )
            }

            Label("구간 자르기")
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                OptionField(o.sectionStart, "시작", "처음", errors["sectionStart"], Modifier.weight(1f)) {
                    onChange(o.copy(sectionStart = it))
                }
                OptionField(o.sectionEnd, "끝", "끝까지", errors["sectionEnd"], Modifier.weight(1f)) {
                    onChange(o.copy(sectionEnd = it))
                }
            }
            Text("1:23, 1:02:03, 83(초) 처럼 적어요. 하나만 적어도 돼요.", color = c.textDim, fontSize = 12.sp)
        }
    }
}

@Composable
private fun Label(text: String) = Text(
    text,
    color = Qm.colors.textDim,
    fontSize = 12.sp,
    fontWeight = FontWeight.Bold,
    modifier = Modifier.padding(top = 6.dp),
)

@Composable
private fun OptionField(
    value: String,
    label: String,
    placeholder: String,
    error: String?,
    modifier: Modifier,
    onChange: (String) -> Unit,
) = Column(modifier) {
    Text(label, color = Qm.colors.textDim, fontSize = 12.sp, modifier = Modifier.padding(start = 6.dp, bottom = 4.dp))
    QmField(
        value,
        onChange,
        placeholder,
        error = error,
        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Ascii, imeAction = ImeAction.Done),
    )
}
