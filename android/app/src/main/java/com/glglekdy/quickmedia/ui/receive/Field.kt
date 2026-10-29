package com.glglekdy.quickmedia.ui.receive

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.glglekdy.quickmedia.ui.theme.Pretendard
import com.glglekdy.quickmedia.ui.theme.Qm

private val Shape = RoundedCornerShape(16.dp)

/** 테두리 없는 입력칸. 링크 칸과 옵션 시트의 칸이 같이 쓴다. 틀리면 빨간 테두리와 아래 문구. */
@Composable
fun QmField(
    value: String,
    onChange: (String) -> Unit,
    placeholder: String,
    modifier: Modifier = Modifier,
    error: String? = null,
    keyboardOptions: KeyboardOptions = KeyboardOptions.Default,
    keyboardActions: KeyboardActions = KeyboardActions.Default,
    trailing: (@Composable () -> Unit)? = null,
) {
    val c = Qm.colors
    Column(modifier) {
        Row(
            Modifier
                .fillMaxWidth()
                .clip(Shape)
                .background(c.field)
                .border(1.5.dp, if (error != null) c.bad else Color.Transparent, Shape)
                .padding(start = 14.dp, end = if (trailing != null) 4.dp else 14.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(Modifier.weight(1f).padding(vertical = 14.dp)) {
                if (value.isEmpty()) Text(placeholder, color = c.textDim, fontSize = 14.sp, maxLines = 1)
                BasicTextField(
                    value = value,
                    onValueChange = onChange,
                    singleLine = true,
                    textStyle = TextStyle(color = c.text, fontSize = 14.sp, fontFamily = Pretendard),
                    cursorBrush = SolidColor(c.accent),
                    keyboardOptions = keyboardOptions,
                    keyboardActions = keyboardActions,
                    modifier = Modifier.fillMaxWidth(),
                )
            }
            trailing?.invoke()
        }
        if (error != null) {
            Text(error, color = c.bad, fontSize = 12.sp, modifier = Modifier.padding(start = 6.dp, top = 4.dp))
        }
    }
}
