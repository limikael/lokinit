package TARGET_PACKAGE_NAME;

import javax.microedition.khronos.egl.EGLConfig;
import javax.microedition.khronos.opengles.GL10;

import android.app.Activity;
import android.os.Bundle;
import android.util.Log;

import android.view.View;
import android.view.Surface;
import android.view.Window;
import android.view.WindowManager.LayoutParams;
import android.view.SurfaceView;
import android.view.SurfaceHolder;
import android.view.MotionEvent;
import android.view.KeyEvent;
import android.view.inputmethod.InputMethodManager;

import android.content.Context;
import android.content.Intent;


import loki_native.LokiNative;

// note: //% is a special lokinit's pre-processor for plugins
// when there are no plugins - //% whatever will be replaced to an empty string
// before compiling

//% IMPORTS

class LokiSurface
    extends
        SurfaceView
    implements
        View.OnTouchListener,
        View.OnKeyListener,
        SurfaceHolder.Callback {

    public LokiSurface(Context context){
        super(context);
        getHolder().addCallback(this);

        setFocusable(true);
        setFocusableInTouchMode(true);
        requestFocus();
        setOnTouchListener(this);
        setOnKeyListener(this);
    }

    @Override
    public void surfaceCreated(SurfaceHolder holder) {
        Log.i("SAPP", "surfaceCreated");
        Surface surface = holder.getSurface();
        LokiNative.surfaceOnSurfaceCreated(surface);
    }

    @Override
    public void surfaceDestroyed(SurfaceHolder holder) {
        Log.i("SAPP", "surfaceDestroyed");
        Surface surface = holder.getSurface();
        LokiNative.surfaceOnSurfaceDestroyed(surface);
    }

    @Override
    public void surfaceChanged(SurfaceHolder holder,
                               int format,
                               int width,
                               int height) {
        Log.i("SAPP", "surfaceChanged");
        Surface surface = holder.getSurface();
        LokiNative.surfaceOnSurfaceChanged(surface, width, height);

    }

    @Override
    public boolean onTouch(View v, MotionEvent event) {
        int pointerCount = event.getPointerCount();
        int action = event.getActionMasked();

        switch(action) {
        case MotionEvent.ACTION_MOVE: {
            for (int i = 0; i < pointerCount; i++) {
                final int id = event.getPointerId(i);
                final float x = event.getX(i);
                final float y = event.getY(i);
                LokiNative.surfaceOnTouch(id, 0, x, y);
            }
            break;
        }
        case MotionEvent.ACTION_UP: {
            final int id = event.getPointerId(0);
            final float x = event.getX(0);
            final float y = event.getY(0);
            LokiNative.surfaceOnTouch(id, 1, x, y);
            break;
        }
        case MotionEvent.ACTION_DOWN: {
            final int id = event.getPointerId(0);
            final float x = event.getX(0);
            final float y = event.getY(0);
            LokiNative.surfaceOnTouch(id, 2, x, y);
            break;
        }
        case MotionEvent.ACTION_POINTER_UP: {
            final int pointerIndex = event.getActionIndex();
            final int id = event.getPointerId(pointerIndex);
            final float x = event.getX(pointerIndex);
            final float y = event.getY(pointerIndex);
            LokiNative.surfaceOnTouch(id, 1, x, y);
            break;
        }
        case MotionEvent.ACTION_POINTER_DOWN: {
            final int pointerIndex = event.getActionIndex();
            final int id = event.getPointerId(pointerIndex);
            final float x = event.getX(pointerIndex);
            final float y = event.getY(pointerIndex);
            LokiNative.surfaceOnTouch(id, 2, x, y);
            break;
        }
        case MotionEvent.ACTION_CANCEL: {
            for (int i = 0; i < pointerCount; i++) {
                final int id = event.getPointerId(i);
                final float x = event.getX(i);
                final float y = event.getY(i);
                LokiNative.surfaceOnTouch(id, 3, x, y);
            }
            break;
        }
        default:
            break;
        }

        return true;
    }

    // docs says getCharacters are deprecated
    // but somehow on non-latyn input all keyCode and all the relevant fields in the KeyEvent are zeros
    // and only getCharacters has some usefull data
    @SuppressWarnings("deprecation")
    @Override
    public boolean onKey(View v, int keyCode, KeyEvent event) {
        if (event.getAction() == KeyEvent.ACTION_DOWN && keyCode != 0) {
            LokiNative.surfaceOnKeyDown(keyCode);
        }

        if (event.getAction() == KeyEvent.ACTION_UP && keyCode != 0) {
            LokiNative.surfaceOnKeyUp(keyCode);
        }
        
        if (event.getAction() == KeyEvent.ACTION_UP || event.getAction() == KeyEvent.ACTION_MULTIPLE) {
            int character = event.getUnicodeChar();
            if (character == 0) {
                String characters = event.getCharacters();
                if (characters != null && characters.length() >= 0) {
                    character = characters.charAt(0);
                }
            }

            if (character != 0) {
                LokiNative.surfaceOnCharacter(character);
            }
        }

        return true;
    }

    public Surface getNativeSurface() {
        return getHolder().getSurface();
    }
}

public class MainActivity extends Activity {
    //% MAIN_ACTIVITY_BODY

    private LokiSurface view;

    static {
        System.loadLibrary("LIBRARY_NAME");
    }

    @Override
    public void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        this.requestWindowFeature(Window.FEATURE_NO_TITLE);

        view = new LokiSurface(this);
        setContentView(view);

        LokiNative.activityOnCreate(this);

        //% MAIN_ACTIVITY_ON_CREATE
    }

    @Override
    protected void onResume() {
        super.onResume();
        LokiNative.activityOnResume();

        //% MAIN_ACTIVITY_ON_RESUME
    }

    @Override
    public void onBackPressed() {
        Log.w("SAPP", "onBackPressed");

        // TODO: here is the place to handle request_quit/order_quit/cancel_quit

        super.onBackPressed();
    }

    @Override
    protected void onStop() {
        super.onStop();
    }

    @Override
    protected void onDestroy() {
        super.onDestroy();

        LokiNative.activityOnDestroy();
    }

    @Override
    protected void onPause() {
        super.onPause();
        LokiNative.activityOnPause();

        //% MAIN_ACTIVITY_ON_PAUSE
    }

    @Override
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        //% MAIN_ACTIVITY_ON_ACTIVITY_RESULT
    }

    public void setFullScreen(final boolean fullscreen) {
        runOnUiThread(new Runnable() {
                @Override
                public void run() {
                    View decorView = getWindow().getDecorView();

                    if (fullscreen) {
                        getWindow().setFlags(LayoutParams.FLAG_LAYOUT_NO_LIMITS, LayoutParams.FLAG_LAYOUT_NO_LIMITS);
                        getWindow().getAttributes().layoutInDisplayCutoutMode = LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES;
                        // Android deprecate dsetSystemUiVisibility, but it is not clear
                        // from the deprecation notes what exaclty we should do instead
                        // hope this works! but just in case, left the old code in a comment below
                        //int uiOptions = View.SYSTEM_UI_FLAG_HIDE_NAVIGATION | View.SYSTEM_UI_FLAG_FULLSCREEN | View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY;
                        //decorView.setSystemUiVisibility(uiOptions);
                        getWindow().setDecorFitsSystemWindows(false);
                    }
                    else {
                       // there might be a bug hidden here: setSystemUiVisibility was dealing with the bars
                        // and setDecorFitsSystemWindows might not, needs some testing
                        //decorView.setSystemUiVisibility(0);
                        getWindow().setDecorFitsSystemWindows(true);

                    }
                }
            });
    }

    public void showKeyboard(final boolean show) {
        runOnUiThread(new Runnable() {
                @Override
                public void run() {
                    if (show) {
                        InputMethodManager imm = (InputMethodManager)getSystemService(Context.INPUT_METHOD_SERVICE);
                        imm.showSoftInput(view, 0);
                    } else {
                        InputMethodManager imm = (InputMethodManager) getSystemService(Context.INPUT_METHOD_SERVICE);
                        imm.hideSoftInputFromWindow(view.getWindowToken(),0); 
                    }
                }
            });
    }
}

