param([string]$Build, [int]$Port=49327, [string]$CaptureDir="$env:TEMP\DentedDestiny-Z7-qa")
if (!$Build) { throw 'Pass -Build with the absolute path of DentedDestiny-Z7.html.' }
$BuildUri=([Uri](Resolve-Path $Build).Path).AbsoluteUri
New-Item -ItemType Directory -Force -Path $CaptureDir|Out-Null
$ErrorActionPreference = 'Stop'
$script:ws = [System.Net.WebSockets.ClientWebSocket]::new()
$targets = Invoke-RestMethod "http://127.0.0.1:$Port/json/list"
$target = $targets | Where-Object { $_.type -eq 'page' } | Select-Object -First 1
$script:ws.ConnectAsync([Uri]$target.webSocketDebuggerUrl,[Threading.CancellationToken]::None).GetAwaiter().GetResult()|Out-Null
$script:seq = 0
function Cdp($method, $params) {
 $script:seq++
 $id = $script:seq
 $payload = @{ id=$id; method=$method; params=$params } | ConvertTo-Json -Depth 40 -Compress
 $bytes = [Text.Encoding]::UTF8.GetBytes($payload)
 $script:ws.SendAsync([ArraySegment[byte]]::new($bytes),[Net.WebSockets.WebSocketMessageType]::Text,$true,[Threading.CancellationToken]::None).GetAwaiter().GetResult()
 while ($true) {
  $stream=[IO.MemoryStream]::new()
  do {
   $buffer=New-Object byte[] 1048576
   $part=$script:ws.ReceiveAsync([ArraySegment[byte]]::new($buffer),[Threading.CancellationToken]::None).GetAwaiter().GetResult()
   $stream.Write($buffer,0,$part.Count)
  } until ($part.EndOfMessage)
  $msg=[Text.Encoding]::UTF8.GetString($stream.ToArray()) | ConvertFrom-Json
  if ($msg.id -eq $id) { if ($msg.error) { throw ($msg.error | ConvertTo-Json) }; return $msg.result }
 }
}
function Eval($expression) { $r=Cdp 'Runtime.evaluate' @{expression=$expression;awaitPromise=$true;returnByValue=$true}; if ($r.exceptionDetails) {throw ($r.exceptionDetails|ConvertTo-Json -Depth 10)}; return $r.result.value }
function Shot($name) {$r=Cdp 'Page.captureScreenshot' @{format='png';captureBeyondViewport=$false};[IO.File]::WriteAllBytes((Join-Path $CaptureDir "$name.png"),[Convert]::FromBase64String($r.data))}
function Key($key,$code,$vk) {Cdp 'Input.dispatchKeyEvent' @{type='keyDown';key=$key;code=$code;windowsVirtualKeyCode=$vk}|Out-Null;Cdp 'Input.dispatchKeyEvent' @{type='keyUp';key=$key;code=$code;windowsVirtualKeyCode=$vk}|Out-Null}

function WaitMode($mode) {
 $js="new Promise((r,j)=>{let n=0;let t=setInterval(()=>{if(window.__DENTED_DIAGNOSTICS__?.mode === '$mode'){clearInterval(t);r(window.__DENTED_DIAGNOSTICS__)}else if(n++>120){clearInterval(t);j('Expected $mode')}},200)})"
 return Eval $js
}
function Pad($button) {
 Eval "window.__testPad.buttons[$button]={pressed:true,touched:true,value:1};new Promise(r=>setTimeout(r,140))"|Out-Null
 Eval "window.__testPad.buttons[$button]={pressed:false,touched:false,value:0};new Promise(r=>setTimeout(r,140))"|Out-Null
}
Cdp 'Page.enable' @{}|Out-Null
Cdp 'Emulation.setDeviceMetricsOverride' @{width=1920;height=1080;deviceScaleFactor=1;mobile=$false}|Out-Null
Cdp 'Page.navigate' @{url=$BuildUri}|Out-Null
Start-Sleep -Milliseconds 1200
WaitMode 'Title'|Out-Null
Eval 'new Promise(r=>setTimeout(r,1800))'|Out-Null
$adapter=Eval 'navigator.gpu.requestAdapter().then(a=>({vendor:a.info.vendor,architecture:a.info.architecture,device:a.info.device,description:a.info.description}))'
Shot 'title'
Cdp 'Input.dispatchMouseEvent' @{type='mouseMoved';x=400;y=680;buttons=0}|Out-Null
# A standard-mapped synthetic pad exercises the browser Gamepad API and gilrs adapter.
Eval "window.__testErrors=[];addEventListener('error',e=>__testErrors.push(e.message));addEventListener('unhandledrejection',e=>__testErrors.push(String(e.reason)));window.__testPad={id:'DualSense Standard Test',index:0,connected:true,mapping:'standard',axes:[0,0,0,0],buttons:Array.from({length:17},()=>({pressed:false,touched:false,value:0}))};Object.defineProperty(navigator,'getGamepads',{value:()=>[{...__testPad,timestamp:performance.now()}]});"|Out-Null
Start-Sleep -Milliseconds 200
Pad 13
if((Eval 'window.__DENTED_DIAGNOSTICS__.menu') -ne 1){throw 'D-pad focus failed'}
Pad 0
WaitMode 'Playing'|Out-Null
Key 'e' 'KeyE' 69
WaitMode 'Rest'|Out-Null
Shot 'sanctuary'
Key 'Backspace' 'Backspace' 8
WaitMode 'Playing'|Out-Null
Pad 5
Start-Sleep -Milliseconds 650
Pad 7
Start-Sleep -Milliseconds 1100
Pad 1
Start-Sleep -Milliseconds 600
Pad 6
Pad 9
WaitMode 'Pause'|Out-Null
Shot 'controller-pause'
$combat=Eval 'window.__DENTED_DIAGNOSTICS__'
if($combat.metrics.lights -lt 1 -or $combat.metrics.heavies -lt 1 -or $combat.metrics.dodges -lt 1){throw 'Controller combat failed'}
Pad 13
Pad 0
WaitMode 'Journal'|Out-Null
Shot 'journal'
Pad 1
WaitMode 'Pause'|Out-Null
Pad 13
Pad 13
Pad 0
WaitMode 'Settings'|Out-Null
Shot 'settings'
# Pointer and keyboard confirmation share one focus model and each activate exactly once.
$aid=Eval '__DENTED_DIAGNOSTICS__.options.assist'
Cdp 'Input.dispatchMouseEvent' @{type='mouseMoved';x=400;y=360;buttons=0}|Out-Null
Cdp 'Input.dispatchMouseEvent' @{type='mousePressed';x=400;y=360;button='left';buttons=1;clickCount=1}|Out-Null
Cdp 'Input.dispatchMouseEvent' @{type='mouseReleased';x=400;y=360;button='left';buttons=0;clickCount=1}|Out-Null
Eval 'new Promise(r=>setTimeout(r,300))'|Out-Null
if((Eval '__DENTED_DIAGNOSTICS__.options.assist') -eq $aid){throw 'Pointer menu activation failed'}
Key 'Enter' 'Enter' 13
Eval 'new Promise(r=>setTimeout(r,300))'|Out-Null
if((Eval '__DENTED_DIAGNOSTICS__.options.assist') -ne $aid){throw 'Keyboard confirmation double-fired or failed after a pointer selection'}
Cdp 'Input.dispatchMouseEvent' @{type='mouseMoved';x=400;y=680;buttons=0}|Out-Null
Pad 0
Pad 1
WaitMode 'Pause'|Out-Null
Pad 13
Pad 13
Pad 13
Pad 13
Pad 0
WaitMode 'Title'|Out-Null
# The untouched title must start autonomous play in real time.
$before=Eval 'JSON.parse(localStorage.getItem("dented-destiny-z7-v1"))'
$run=Eval 'new Promise((resolve,reject)=>{let n=0;let t=setInterval(()=>{if(window.__DENTED_DIAGNOSTICS__?.demo){clearInterval(t);resolve(window.__DENTED_DIAGNOSTICS__)}else if(n++>100){clearInterval(t);reject("Attract did not start")}},200)})'
$rows=@();$milestones=@{}
for($i=0;$i -lt 190;$i++) {
 $d=Eval 'new Promise(r=>setTimeout(()=>r(window.__DENTED_DIAGNOSTICS__),1000))'
 $rows += $d
 if($d.boss_active -and !$milestones.boss){Shot 'boss';$milestones.boss=$true}
 if($d.metrics.phase_two -and !$milestones.phase){Shot 'boss-phase-two';$milestones.phase=$true}
 if($d.mode -eq 'Victory'){Shot 'victory';$milestones.victory=$true;break}
}
$after=Eval 'JSON.parse(localStorage.getItem("dented-destiny-z7-v1"))'
if(($before|ConvertTo-Json -Depth 20 -Compress) -ne ($after|ConvertTo-Json -Depth 20 -Compress)){throw 'Attract changed the personal save'}
# The pointer was placed over the victory menu before play. A click must only
# cancel attract, without the release accidentally activating the title's same row.
Cdp 'Input.dispatchMouseEvent' @{type='mousePressed';x=400;y=680;button='left';buttons=1;clickCount=1}|Out-Null
Cdp 'Input.dispatchMouseEvent' @{type='mouseReleased';x=400;y=680;button='left';buttons=0;clickCount=1}|Out-Null
WaitMode 'Title'|Out-Null
$cancelled=Eval 'new Promise(r=>setTimeout(()=>r(__DENTED_DIAGNOSTICS__.mode),700))'
if($cancelled -ne 'Title'){throw 'Demo click activated a title menu action'}
$errors=Eval 'window.__testErrors'
$report=@{adapter=$adapter;combat=$combat;milestones=$milestones;frames=$rows;errors=$errors;performance=($rows|Measure-Object -Property fps -Average -Minimum -Maximum);save_isolated=$true;input_cancelled=$true}
$report|ConvertTo-Json -Depth 20|Set-Content (Join-Path $CaptureDir 'report.json')
if(!$milestones.victory){throw 'No rendered victory in the allowed run'}
if($report.performance.Average -lt 25){throw 'Average performance below 25 FPS'}
if($errors.Count -gt 0){throw 'Browser errors recorded'}
$report.performance|ConvertTo-Json -Compress
$script:ws.Dispose()
