document.addEventListener('DOMContentLoaded', () => {
    // --- DOM Elements ---
    const usernameInput = document.getElementById('username');
    const loadUserBtn = document.getElementById('load-user-btn');
    const mainControls = document.getElementById('main-controls');
    
    const cycleSelect = document.getElementById('cycle-select');
    const waveSelect = document.getElementById('wave-select');
    const weekSelect = document.getElementById('week-select');
    
    const bpInput = document.getElementById('bench_press');
    const sqInput = document.getElementById('squat');
    const dlInput = document.getElementById('deadlift');
    const ohpInput = document.getElementById('overhead_press');
    const saveLiftsBtn = document.getElementById('save-lifts-btn');

    const amrapSection = document.getElementById('amrap-section');
    const programOutput = document.getElementById('program-output');

    // --- App State ---
    let currentUser = null;
    let liftHistory = {};
    let amrapHistory = {};

    // --- Event Listeners ---
    loadUserBtn.addEventListener('click', handleLoadUser);
    saveLiftsBtn.addEventListener('click', handleSaveLifts);
    [cycleSelect, waveSelect, weekSelect].forEach(el => el.addEventListener('change', displayProgram));
    [cycleSelect, waveSelect].forEach(el => el.addEventListener('change', updateLiftInputsForSelectedWave));

    // 초기 버튼 텍스트 설정
    loadUserBtn.textContent = '사용자 등록하기';

    // --- Functions ---

    /**
     * 사용자 정보를 불러오고 UI를 초기화
     */
    async function handleLoadUser() {
        const username = usernameInput.value.trim();
        if (!username) {
            alert('사용자 이름을 입력하세요.');
            return;
        }
        currentUser = username;
        try {
            const response = await fetch(`/api/user/${username}`);
            if (response.ok) {
                const user = await response.json();
                liftHistory = user.lift_history || {};
                amrapHistory = user.amrap_history || {}; // amrapHistory도 초기화
                alert(`${username}님의 데이터를 불러왔습니다.`);
                // 사용자 로드 후 버튼 텍스트 변경
                loadUserBtn.textContent = '다른 사용자 불러오기';
            } else if (response.status === 404) {
                // 새 사용자인 경우
                liftHistory = {};
                amrapHistory = {};
                console.log('New user. Ready to register.');
                // 새 사용자 등록을 위해 버튼 텍스트 변경
                loadUserBtn.textContent = '다른 사용자 불러오기';
            } else {
                // 404 이외의 다른 서버 오류
                throw new Error(`서버 응답 오류: ${response.status} ${response.statusText}`);
            }
            initializeUI();
        } catch (error) {
            console.error('Error:', error);
            alert(`데이터를 불러오는 중 오류가 발생했습니다: ${error.message}`);
        }
    }

    /**
     * UI 컨트롤을 보이게 하고 사이클 셀렉트 박스를 채움
     */
    function initializeUI() {
        // 사이클 번호 채우기 (아직 안 채워졌을 경우에만)
        if (cycleSelect.options.length === 0) {
            for (let i = 1; i <= 10; i++) {
                cycleSelect.innerHTML += `<option value="${i}">${i}번째 사이클</option>`;
            }
        }
        mainControls.classList.remove('hidden');
        updateLiftInputsForSelectedWave();
        displayProgram();
    }

    /**
     * 선택된 Wave에 해당하는 3RM 값을 입력 필드에 표시
     */
    function updateLiftInputsForSelectedWave() {
        const waveKey = `${cycleSelect.value}-${waveSelect.value}s`;
        const lifts = liftHistory[waveKey];

        bpInput.value = lifts?.bench_press || 0;
        sqInput.value = lifts?.squat || 0;
        dlInput.value = lifts?.deadlift || 0;
        ohpInput.value = lifts?.overhead_press || 0;
    }

    /**
     * 현재 입력된 3RM 값을 서버에 저장
     */
    async function handleSaveLifts() {
        if (!currentUser) {
            alert('먼저 사용자 이름을 불러오세요.');
            return;
        }
        const waveKey = `${cycleSelect.value}-${waveSelect.value}s`;
        const lifts = {
            bench_press: parseFloat(bpInput.value) || 0,
            squat: parseFloat(sqInput.value) || 0,
            deadlift: parseFloat(dlInput.value) || 0,
            overhead_press: parseFloat(ohpInput.value) || 0,
        };

        // 로컬 상태 먼저 업데이트
        liftHistory[waveKey] = lifts;

        // 변경된 핸들러에 맞게 payload 구성 (두 번째 문제 해결)
        const payload = {
            waveKey: waveKey,
            lifts: lifts,
        };

        try {
            const response = await fetch(`/api/user/${currentUser}`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload) 
            });
            if (response.ok) {
                alert(`[${waveKey}] Wave의 3RM이 저장되었습니다.`);
                displayProgram(); // 저장 후 프로그램 다시 표시
            } else {
                throw new Error(`서버 응답 오류: ${response.status} ${response.statusText}`);
            }
        } catch (error) {
            console.error('Error saving lifts:', error);
            alert(`데이터 저장 중 오류 발생: ${error.message}`);
        }
    }

    /**
     * 선택된 주차의 운동 프로그램을 서버에 요청하고 화면에 표시
     */
    async function displayProgram() {
        if (!currentUser) return;

        const cycle = cycleSelect.value;
        const wave = waveSelect.value;
        const week = weekSelect.value;
        const waveKey = `${cycle}-${wave}s`;

        // 실현주(3주차)가 아니면 AMRAP 섹션 숨기기
        if (week !== '3') {
            amrapSection.classList.add('hidden');
        }

        if (!liftHistory[waveKey]) {
            programOutput.innerHTML = `<p style="color: orange;">[${waveKey}] Wave의 3RM 데이터가 없습니다. 입력 후 저장해주세요.</p>`;
            return;
        }

        try {
            const url = `/api/program/${currentUser}?cycle=${cycle}&wave=${wave}&week=${week}`;
            const response = await fetch(url);
            if (!response.ok) {
                throw new Error(`서버 응답 오류: ${response.status} ${response.statusText}`);
            }
            const programWeek = await response.json();
            renderProgram(programWeek);
        } catch (error) {
            console.error('Error fetching program:', error);
            programOutput.innerHTML = `<p style="color: red;">프로그램을 불러오는 중 오류 발생: ${error.message}</p>`;
        }
    }

    /**
     * 한 주의 프로그램을 HTML로 렌더링
     */
    function renderProgram(week) {
        programOutput.innerHTML = '';
        const weekDiv = document.createElement('div');
        weekDiv.className = 'program-week';

        // 실현주(3주차)인 경우 AMRAP 입력 UI 렌더링
        if (week.week_in_wave === 3) {
            renderAmrapSection();
        }

        let weekTitle = `<h3>${week.wave_type} (${week.week_in_wave}주차)</h3>`;
        weekDiv.innerHTML = weekTitle;

        week.days.forEach(day => {
            const dayDiv = document.createElement('div');
            dayDiv.className = 'program-day';
            let setsHtml = day.sets.map(set => {
                const mainSetText = `<li>${set.weight}kg &times; ${set.reps === -1 ? 'AMRAP' : set.reps + ' reps'}</li>`;
                const formulaText = `<li class="set-formula">(WM의 ${Math.round(set.percentage * 100)}%)</li>`;
                return mainSetText + formulaText;
            }).join('');

            dayDiv.innerHTML = `
                <h4>${day.lift_name}</h4>
                <ul>${setsHtml}</ul>
            `;
            weekDiv.appendChild(dayDiv);
        });
        programOutput.appendChild(weekDiv);
    }

    /**
     * AMRAP 입력 섹션을 렌더링
     */
    function renderAmrapSection() {
        const waveKey = `${cycleSelect.value}-${waveSelect.value}s`;
        const savedAmraps = amrapHistory[waveKey] || {
            bench_press: 0,
            squat: 0,
            deadlift: 0,
            overhead_press: 0,
        };

        amrapSection.classList.remove('hidden');
        amrapSection.innerHTML = `
            <h3>실현주(3주차) AMRAP 결과 입력</h3>
            <p>각 운동의 마지막 AMRAP 세트에서 성공한 횟수를 입력하세요.</p>
            <div class="lift-input-row">
                <label>벤치프레스 AMRAP 횟수:</label> <input type="number" id="amrap-bench_press" value="${savedAmraps.bench_press}">
            </div>
            <div class="lift-input-row">
                <label>스쿼트 AMRAP 횟수:</label> <input type="number" id="amrap-squat" value="${savedAmraps.squat}">
            </div>
            <div class="lift-input-row">
                <label>데드리프트 AMRAP 횟수:</label> <input type="number" id="amrap-deadlift" value="${savedAmraps.deadlift}">
            </div>
            <div class="lift-input-row">
                <label>오버헤드프레스 AMRAP 횟수:</label> <input type="number" id="amrap-overhead_press" value="${savedAmraps.overhead_press}">
            </div>
            <button id="calculate-next-wave-btn">다음 Wave 3RM 계산 및 저장</button>
        `;

        document.getElementById('calculate-next-wave-btn').addEventListener('click', handleCalculateNextWave);
    }

    /**
     * '다음 Wave 계산' 버튼 클릭 시 실행될 함수
     */
    async function handleCalculateNextWave() {
        const payload = {
            cycle: parseInt(cycleSelect.value),
            wave: parseInt(waveSelect.value),
            amrap_reps: {
                bench_press: parseInt(document.getElementById('amrap-bench_press').value) || 0,
                squat: parseInt(document.getElementById('amrap-squat').value) || 0,
                deadlift: parseInt(document.getElementById('amrap-deadlift').value) || 0,
                overhead_press: parseInt(document.getElementById('amrap-overhead_press').value) || 0,
            }
        };

        try {
            const response = await fetch(`/api/amrap/${currentUser}`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload)
            });

            if (!response.ok) {
                throw new Error(`서버 응답 오류: ${response.status} ${response.statusText}`);
            }

            const updatedUser = await response.json();
            liftHistory = updatedUser.lift_history; // 로컬 liftHistory 갱신
            amrapHistory = updatedUser.amrap_history; // 로컬 amrapHistory 갱신
            alert('다음 Wave의 3RM이 계산되어 저장되었습니다! 다음 Wave를 선택하여 확인하세요.');
            
        } catch (error) {
            console.error('Error calculating next wave:', error);
            alert(`AMRAP 결과 저장 중 오류 발생: ${error.message}`);
        }
    }
});
